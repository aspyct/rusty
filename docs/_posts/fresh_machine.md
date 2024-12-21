---
layout: post
title: "10 minutes in, they tried to hack me"
---

Ever wondered what happens to your server once it goes online? You may find it both amusing and scary.

Long story short: within 10 minutes, someone will attempt to gain root access. Whoo!

Let's dig into it. If you have basic linux skills (bash, mostly), you should be able to follow along and replicate the same experiment on your server.

## Setup

For this first experiment, I decided to keep things simple: boot up a new virtual machine in the cloud, don't worry about DNS or anything, and log 3 things:

1. port scans
2. ssh connection attempts
3. http connections

I booted up the machine, upgraded software, added a firewall, apache and tcpdump.

```
# apt update && apt upgrade
# reboot
# apt install tcpdump ufw apache2
# ufw allow ssh
# ufw allow http
# ufw enable
```

## To war!

Or rather: To sitting idle and seeing things come our way...

Apache and sshd are collecting their own logs. All I need to do manually is log the TCP SYN packets sent to my machine with `tcpdump`.

## TCP SYN packets

Let's start with the TCP SYN packets received.
A SYN packet is the "hello" between machines. If my machine wants to establish a connection to yours, the first step is sending one of those SYN packets.

You can read more about the [TCP handshake](https://en.wikipedia.org/wiki/Transmission_Control_Protocol#Connection_establishment) on wikipedia.

```
# tcpdump "tcp[tcpflags] & tcp-syn == tcp-syn"
10:20:13.226447 IP 79.110.62.244.55430 > <my_server>.16575: Flags [S]
10:20:15.199621 IP scan.cypex.ai.59856 > <my_server>.49169: Flags [S]
10:20:15.845191 IP 169.254.169.254.http > <my_server>.33456: Flags [S.]
10:20:17.917878 IP 79.110.62.244.55430 > <my_server>.21094: Flags [S]
10:20:18.017210 IP 207.90.244.15.23320 > <my_server>.8139: Flags [S]
10:20:21.938249 IP 185.7.214.52.42179 > <my_server>.33089: Flags [S]
10:20:25.101734 IP 146.103.45.185.55292 > <my_server>.http-alt: Flags [S]
10:20:42.397456 IP 79.110.62.242.55429 > <my_server>.13671: Flags [S]
10:20:42.961234 IP ip-112-6.4vendeta.com.55441 > <my_server>.60521: Flags [S]
10:20:54.168294 IP 205.210.31.180.62428 > <my_server>.ssh: Flags [S]
10:20:54.168360 IP <my_server>.ssh > 205.210.31.180.62428: Flags [S.]

[...]

The complete file is 2810 lines long (3 hours of logs).
```

Each line above represents a machine trying to connect to my server, or checking if the port is open (for example with tools like [nmap](https://nmap.org)).

For example, the first line indicates that the machine with IP address `79.110.62.244` tried to connect to my server on port 16575 at 10:20:13 UTC.

The second line shows a hostname for the sender, instead of an IP address: `tcpdump` does a reverse DNS query for each line and tries to come up with a name for the sender, instead of just an IP address.

Finally, the last two lines are someone trying to connect to my SSH server. We will find that later in our SSH logs.

## It's a SYN!

Here's a quick breakdown of the IPs/hosts I've seen the most in my log file:

**Please note: reverse DNS lookup [cannot be trusted](https://security.stackexchange.com/questions/257426/can-this-logic-with-regard-to-checking-reverse-dns-records-be-flawed). The data that appears below make be entirely wrong, for all I know.**

```
# Roughly based on the output of this command:
$ cut -d ' ' -f 3 tcpdump-syn.log | rev | cut -d. -f2- | sort | rev | uniq -c

79.110.62.244
79.110.62.242
80.94.95.176
*.shadowserver.com
*.stretchoid.com
*.bc.googleusercontent.com
*.ip.linodeusercontent.com
*.internet-research-project.net

[...]
```

Now, an IP scan isn't really an attack per se. They're just trying to find out if I have something running on a given port.
There are legitimates reasons why you would scan a host for open ports, but it's also a means to do reconnaissance for future attacks.

One of the well-known names that showed up in the list is [Shodan](https://www.shodan.io/dashboard). Shodan is a search engine that is commonly used
for recon prior to a vulnerability enumeration.

Shadowserver is possibly related to [shadowserver.org](https://www.shadowserver.org). They are apparently funded by the UK government, and offer a [public dashboard](https://dashboard.shadowserver.org/)
with a fair amount of data available on it.

I'm mostly fine with these two above, since they're offering a public service and are of value for my security assessments.

Others seem more obscure, and don't say what they use that data for, or flat out don't exist online. As for the IP addresses, well... I can only assume they're trying find vulnerabilities.

## SSH logs

I didn't even have time to start `tcpdump` that already something was trying to connect to my SSH server!

```
# journalctl -u ssh.service
Dec 21 10:14:03 <my_server> sshd[1292]: Accepted publickey for root from <my_ip> port 62192 ssh2: RSA SHA256:<redacted>
Dec 21 10:19:43 <my_server> sshd[2067]: error: kex_exchange_identification: Connection closed by remote host
Dec 21 10:19:43 <my_server> sshd[2067]: Connection closed by 117.33.249.211 port 44966
Dec 21 10:20:59 <my_server> sshd[2077]: Connection reset by 205.210.31.180 port 62428 [preauth]
Dec 21 10:28:37 <my_server> sshd[2109]: Connection closed by authenticating user root 8.217.134.158 port 36542 [preauth]
```

The first line here is me, connecting with my private key to the server. The rest... isn't me.

Remember those last two lines in the `tcpdump` logs? They are the SSH connection attempt by `205.210.31.180`.
Over the course of 3 hours, roughly 300 connection attempts were made:

- `root` was tried 42 times, the first attempt a measly 10 minutes into the life of the server
- `a3user` is also a popular username, apparently a [default login on IBM software](https://www.rapid7.com/db/modules/exploit/linux/ssh/ibm_drm_a3user/)
- `dolphinscheduler` appeared 4 times. Possibly related to a [Apache Dolphin Scheduler](https://dolphinscheduler.apache.org/)
- `server`, `admin`, `foo` and other generic names are common
- `ISANTA`. Merry Christmas, I guess?
- `gerald`, `maxime`, `gdiaz` and other names appear. Not sure what they're linked to

You can also spot a few key exchange protocol errors, probably someone trying an old vulnerability:

```
Dec 21 11:17:04 <my_server> sshd[2233]: error: kex_protocol_error: type 20 seq 2 [preauth]
Dec 21 11:17:04 <my_server> sshd[2233]: error: kex_protocol_error: type 30 seq 3 [preauth]
Dec 21 11:17:06 <my_server> sshd[2233]: error: kex_protocol_error: type 20 seq 4 [preauth]
Dec 21 11:17:06 <my_server> sshd[2233]: error: kex_protocol_error: type 30 seq 5 [preauth]
Dec 21 11:17:08 <my_server> sshd[2233]: error: kex_protocol_error: type 20 seq 6 [preauth]
Dec 21 11:17:08 <my_server> sshd[2233]: error: kex_protocol_error: type 30 seq 7 [preauth]

[...]

Dec 21 12:13:17 <my_server> sshd[2673]: error: kex_exchange_identification: banner line contains invalid characters

[...]
```

Maybe later I'll setup machines with a password root login, to see how long it lasts...

## Apache access logs

The smallest log file is that of apache (located at `/var/log/apache/access.log`).
To be honest, I expected more entries here, having monitored other websites before.
I guess the machine was too hidden for this to have a big impact.

Still, some requests were made:

```
# Log format:
# IP - - [timestamp] "method uri version" response_code content-size "referer" "user-agent"
78.140.21.51 - - [21/Dec/2024:11:07:24 +0000] "GET / HTTP/1.1" 200 10956 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/601.7.7 (KHTML, like Gecko) Version/9.1.2 Safari/601.7.7"
5.32.176.116 - - [21/Dec/2024:11:37:38 +0000] "GET / HTTP/1.1" 200 10956 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/601.7.7 (KHTML, like Gecko) Version/9.1.2 Safari/601.7.7"
95.214.55.74 - - [21/Dec/2024:11:57:55 +0000] "GET /cgi-bin/luci/;stok=/locale HTTP/1.1" 404 435 "-" "-"
95.214.55.79 - - [21/Dec/2024:11:59:17 +0000] "GET / HTTP/1.1" 200 3380 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.85 Safari/537.36 Edg/90.0.818.46"
185.191.126.213 - - [21/Dec/2024:12:03:47 +0000] "GET / HTTP/1.1" 200 10956 "-" "-"
95.214.55.32 - - [21/Dec/2024:12:07:14 +0000] "GET / HTTP/1.1" 200 10956 "-" "-"
222.228.81.22 - - [21/Dec/2024:12:40:26 +0000] "GET / HTTP/1.1" 200 10956 "-" "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36"
181.49.205.58 - - [21/Dec/2024:12:48:33 +0000] "GET /admin/assets/js/views/login.js HTTP/1.0" 404 454 "-" "xfa1"
95.214.53.205 - - [21/Dec/2024:12:53:51 +0000] "GET / HTTP/1.1" 200 3380 "-" "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.85 Safari/537.36 Edg/90.0.818.46"
203.223.44.74 - - [21/Dec/2024:13:20:09 +0000] "GET / HTTP/1.1" 200 10956 "-" "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.103 Safari/537.36"
141.98.11.155 - - [21/Dec/2024:13:26:26 +0000] "GET / HTTP/1.1" 200 10956 "-" "-"
```

Nothing very fancy. One that seems to be looking for [TP-Link wifi routers](https://www.greynoise.io/blog/active-exploitation-attempts-cve-2023-1389-against-tp-link-archer-gigabit-internet-routers),
and another for [Sangoma FreePBX](https://www.radware.com/blog/security/the-top-web-service-exploits-in-2020/). Not sure what the rest of them were up to.

## Conclusion

Internet is a wild place! Put a vulnerable machine online and it'll be hacked within hours at most.

Vulnerabilities come in all shapes and forms, from out-of-date software to default logins, without forgetting bad passwords and misconfigured web servers.
In fact, even up-to-date software can contain vulnerabilities, also called [zero-days](https://en.wikipedia.org/wiki/Zero-day_vulnerability).

Thankfully, foundational tools like the ssh server and various http servers are well surveyed and rather safe,
but you should keep your machines updated and watch your logs!
