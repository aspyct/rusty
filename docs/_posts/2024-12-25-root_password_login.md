---
layout: post
title: "Painting a big red target on my server"
---

During our [previous experiment]({% post_url 2024-12-22-fresh_machine %}), we saw a total of 300 SSH login attempts within 3 hours of spinning up a new server. That was with key-only ssh login.

Let's paint a big red target on my server and enable password login!

```
# journalctl -u ssh.service
Dec 25 19:58:57 <hostname> systemd[1]: Starting ssh.service - OpenBSD Secure Shell server...
Dec 25 19:58:58 <hostname> sshd[471]: Server listening on 0.0.0.0 port 22.
Dec 25 19:58:58 <hostname> sshd[471]: Server listening on :: port 22.
Dec 25 19:58:58 <hostname> systemd[1]: Started ssh.service - OpenBSD Secure Shell server.
Dec 25 19:59:10 <hostname> sshd[740]: Accepted password for root from <my_ip> port 58789 ssh2 <- that's me
Dec 25 19:59:10 <hostname> sshd[740]: pam_unix(sshd:session): session opened for user root(uid=0) by (uid=0)
Dec 25 19:59:10 <hostname> sshd[740]: pam_env(sshd:session): deprecated reading of user environment enabled
Dec 25 19:59:21 <hostname> sshd[771]: pam_unix(sshd:auth): authentication failure; logname= uid=0 euid=0 tty=ssh ruser= rhost=218.92.0.151  user=root
Dec 25 19:59:22 <hostname> sshd[771]: Failed password for root from 218.92.0.151 port 49558 ssh2 <- that's not me
Dec 25 19:59:25 <hostname> sshd[771]: Failed password for root from 218.92.0.151 port 49558 ssh2
Dec 25 19:59:28 <hostname> sshd[771]: Failed password for root from 218.92.0.151 port 49558 ssh2
Dec 25 19:59:29 <hostname> sshd[771]: Received disconnect from 218.92.0.151 port 49558:11:  [preauth]
Dec 25 19:59:29 <hostname> sshd[771]: Disconnected from authenticating user root 218.92.0.151 port 49558 [preauth]
Dec 25 19:59:29 <hostname> sshd[771]: PAM 2 more authentication failures; logname= uid=0 euid=0 tty=ssh ruser= rhost=218.92.0.151  user=root
```

Machine was spun up around 19:58, and at 19:59:22 somebody tried a password root login. Didn't even take 2 minutes!

Fast forward 2 hours, and there were over 1300 login attempts. Let's pull some metrics from that.

## Analyzing the data

We'll use `rsec` and `duckdb` to analyze the logs. Here's how to install the tools:

```
# apt install build-essential unzip
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# cargo install --version 0.2.0 rsec
# wget https://github.com/duckdb/duckdb/releases/download/v1.1.3/duckdb_cli-linux-amd64.zip
# unzip duckdb_cli-linux-amd64.zip
# mv duckdb /usr/local/bin/
# rm duckdb_cli-linux-amd64.zip
```

Fisrt, let's use `rsec` to parse the failed logins to json:

```
# sudo journalctl -u ssh.service | rsec ssh > failed_logins.ndjson
$ head failed_logins.ndjson
{"time": "Dec 25 19:59:22", "hostname": "<hostname>", "pid": "771", "username": "root", "ip": "218.92.0.151", "port": "49558"}
{"time": "Dec 25 19:59:25", "hostname": "<hostname>", "pid": "771", "username": "root", "ip": "218.92.0.151", "port": "49558"}
{"time": "Dec 25 19:59:28", "hostname": "<hostname>", "pid": "771", "username": "root", "ip": "218.92.0.151", "port": "49558"}
{"time": "Dec 25 20:01:08", "hostname": "<hostname>", "pid": "833", "username": "root", "ip": "218.92.0.151", "port": "57539"}
{"time": "Dec 25 20:01:11", "hostname": "<hostname>", "pid": "833", "username": "root", "ip": "218.92.0.151", "port": "57539"}
{"time": "Dec 25 20:01:14", "hostname": "<hostname>", "pid": "833", "username": "root", "ip": "218.92.0.151", "port": "57539"}
{"time": "Dec 25 20:02:54", "hostname": "<hostname>", "pid": "837", "username": "root", "ip": "218.92.0.151", "port": "12305"}
{"time": "Dec 25 20:02:57", "hostname": "<hostname>", "pid": "837", "username": "root", "ip": "218.92.0.151", "port": "12305"}
{"time": "Dec 25 20:03:00", "hostname": "<hostname>", "pid": "837", "username": "root", "ip": "218.92.0.151", "port": "12305"}
{"time": "Dec 25 20:04:42", "hostname": "<hostname>", "pid": "840", "username": "root", "ip": "218.92.0.151", "port": "17690"}
```

We can import that ndjson file (newline-delimited JSON) into [duckdb](https://duckdb.org/) for further analysis.

```
$ duckdb
D create table failed_logins as select * from read_json_auto('failed_logins.ndjson');
D select count(*) as failed_login_attempts, min(time) as first_attempt, max(time) as last_attempt from failed_logins;
┌───────────────────────┬─────────────────┬─────────────────┐
│ failed_login_attempts │  first_attempt  │  last_attempt   │
│         int64         │     varchar     │     varchar     │
├───────────────────────┼─────────────────┼─────────────────┤
│                  1369 │ Dec 25 19:59:22 │ Dec 25 22:12:49 │
└───────────────────────┴─────────────────┴─────────────────┘

D select ip
       , count(*) as failed_login_count
       , count(distinct username) as distinct_username_count
       , min(time) as first_attempt_at
       , max(time) as last_attempt_at
  from failed_logins
  group by 1 order by 2 desc;
┌────────────────┬────────────────────┬─────────────────────────┬──────────────────┬─────────────────┐
│       ip       │ failed_login_count │ distinct_username_count │ first_attempt_at │ last_attempt_at │
│    varchar     │       int64        │          int64          │     varchar      │     varchar     │
├────────────────┼────────────────────┼─────────────────────────┼──────────────────┼─────────────────┤
│ 137.184.84.118 │                570 │                     141 │ Dec 25 20:10:39  │ Dec 25 21:02:13 │
│ 156.238.99.83  │                412 │                       7 │ Dec 25 21:31:41  │ Dec 25 21:58:49 │
│ 209.38.30.23   │                239 │                     109 │ Dec 25 20:21:50  │ Dec 25 20:41:29 │
│ 218.92.0.151   │                148 │                       1 │ Dec 25 19:59:22  │ Dec 25 22:12:49 │
└────────────────┴────────────────────┴─────────────────────────┴──────────────────┴─────────────────┘
```

So 4 different IPs tried to login a total of 1369 times from 19:59 to 22:12. Sweet, sweet data!

We already know by now that `218.92.0.151` is trying the `root` user, probably slowly bruteforcing the password, going for the win. I wonder who else is trying to get the root account.

```
D select ip
       , count(*) as attempts
  from failed_logins
  where username = 'root'
  group by 1 order by 2 desc;
┌────────────────┬──────────┐
│       ip       │ attempts │
│    varchar     │  int64   │
├────────────────┼──────────┤
│ 218.92.0.151   │      148 │
│ 137.184.84.118 │       93 │
│ 156.238.99.83  │       82 │
│ 209.38.30.23   │       57 │
└────────────────┴──────────┘
```

Well, everyone's trying root. Hey, why not, go big or go home!

How many other users are we trying, and what's the username?

```
D select count(distinct username) as distinct_usernames from failed_logins;
┌────────────────────┐
│ distinct_usernames │
│       int64        │
├────────────────────┤
│                182 │
└────────────────────┘

D select username
       , count(*)
  from failed_logins
  group by 1 order by 2 desc
  limit 15;
┌──────────────────┬──────────────┐
│     username     │ count_star() │
│     varchar      │    int64     │
├──────────────────┼──────────────┤
│ root             │          380 │
│ ubuntu           │          103 │
│ admin            │           99 │
│ user             │           92 │
│ debian           │           85 │
│ steam            │           17 │
│ test             │           14 │
│ hadoop           │           13 │
│ dolphinscheduler │           12 │
│ oracle           │           11 │
│ redis            │           10 │
│ postgres         │            9 │
│ solana           │            9 │
│ elasticsearch    │            9 │
│ palworld         │            8 │
├──────────────────┴──────────────┤
│ 15 rows               2 columns │
└─────────────────────────────────┘
```

Funny to see `steam` in there. And `palworld` too. This raises a lot of questions! For the curious, I'm including the complete list of usernames at the end of this post.

At this point, I wish I had the list of passwords they tried too. That could be a subject for later :) If anyone knows how to log that in the ssh journal, I'm all ears!

I also thought about creating each of those users with a locked password, but after a quick test, it doesn't look like they can tell if the user exists or not just by trying a login. So creating the users probably wouldn't affect the attempts we're getting here.

## Conclusion

It's probably better to disable password login for SSH, if only to save your poor filesystem from log abuse.

Jokes aside, with key-only login, we had around 100 attempts per hour. With passwords enabled, that bumped up to over 600 attempts per hour. And that's for a fresh server. If you have weak passwords, you are bound to be hacked into sooner or later.

I would also take the precaution to whitelist the users who can login, if applicable. That way, you won't unknowingly open an access when installing a service with a bad default security configuration.

You can use `AllowUsers` for that in your sshd_config file. See `man sshd_config` for more details.

> AllowUsers
>
>  This keyword can be followed by a list of user name patterns, separated by spaces. If specified, login is allowed only for user names that match one of the patterns. [...]

## Annex: Complete list of usernames

```
$ jq -r .username failed_logins.ndjson | sort | uniq -c | sort -nr
 380 root
 103 ubuntu
  99 admin
  92 user
  85 debian
  17 steam
  14 test
  13 hadoop
  12 dolphinscheduler
  11 oracle
  10 redis
   9 solana
   9 postgres
   9 elasticsearch
   8 www
   8 vagrant
   8 sol
   8 palworld
   8 es
   7 www-data
   7 opc
   7 nagios
   7 jito
   7 git
   7 ftpuser
   7 ftp
   7 centos
   6 wordpress
   6 testuser
   6 samba
   6 puppet
   6 nginx
   6 mysql
   6 guest
   6 gitlab
   6 esuser
   6 dev
   6 caddy
   5 wang
   5 uftp
   5 tom
   5 sonar
   5 sftp
   5 satisfactory
   5 ranger
   5 omsagent
   5 node
   5 lighthouse
   5 elastic
   5 ds
   5 drupal
   5 dolphin
   5 docker
   5 deploy
   5 demo
   5 app
   5 apache
   5 amp
   4 vps
   4 uucp
   4 user1
   4 tomcat
   4 terraria
   4 solr
   4 plex
   4 oscar
   4 odoo
   4 minecraft
   4 master
   4 mapr
   4 latitude
   4 jack
   4 grafana
   4 gitlab-runner
   4 factorio
   4 ec2-user
   4 developer
   3 zabbix
   3 vbox
   3 sysadmin
   3 sys
   3 rancher
   3 pi
   3 nvidia
   3 nexus
   3 ldap
   3 kodi
   3 elsearch
   3 bin
   3 arkserver
   3 ark
   3 airflow
   2 zookeeper
   2 yarn
   2 worker
   2 weblogic
   2 virtualbox
   2 user2
   2 ts
   2 server
   2 pal
   2 mongodb
   2 lsb
   2 kubernetes
   2 kingbase
   2 jupyter
   2 jumpserver
   2 jito-validator
   2 jenkins
   2 hive
   2 gpadmin
   2 gmod
   2 gitlab-psql
   2 fil
   2 ethnode
   2 esroot
   2 elk
   2 dstserver
   2 dst
   2 dmdba
   2 data
   2 bot
   2 bigdata
   2 appuser
   2 amandabackup
   1 yealink
   1 wso2
   1 vnc
   1 username
   1 ubnt
   1 tools
   1 test2
   1 system
   1 subsonic
   1 stream
   1 sftpuser
   1 sadmin
   1 runner
   1 red5
   1 proxy
   1 plexserver
   1 openvpn
   1 odoo17
   1 odoo16
   1 observer
   1 niaoyun
   1 nft
   1 mongo
   1 mehdi
   1 media
   1 madsonic
   1 lsfadmin
   1 jms
   1 jfedu1
   1 jellyfin
   1 hikvision
   1 gpuadmin
   1 goeth
   1 gmodserver
   1 gerbera
   1 gbase
   1 g
   1 flussonic
   1 flink
   1 flask
   1 fastuser
   1 esearch
   1 esadmin
   1 emby
   1 dspace
   1 deployer
   1 deepspeed
   1 chain
   1 blockchain
   1 backup
   1 azureuser
   1 awsgui
   1 ansible
   1 ampache
   1 amir
   1 amanda
   1 administrator
```