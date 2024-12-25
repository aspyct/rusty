---
layout: default
---

Rusty-security, aka `rsec` is a set of poorly put together tools to analyze the security events of your linux servers.

![Rusty](/assets/images/rsec.jpg)

Install it with `cargo`:

```
cargo install rsec
```

It's very early alpha for now so don't expect it to be stable, but it should be somewhat useful. You can find usage examples through fun experiments in the posts below.

<h2>Experiments</h2>
<ul>
  {% for post in site.posts %}
    <li>
      <a href="{{ post.url }}">{{ post.date | date_to_long_string }} - {{ post.title }}</a>
    </li>
  {% endfor %}
</ul>
