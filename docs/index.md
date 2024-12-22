---
layout: default
---

Rusty is a set of poorly put together tools to analyze the security events of your linux servers.

It's v0.0.0 for now so I wouldn't rely on it too much. Still, it comes with fun documentation and experiments, so it's worth looking into.

<h2>Experiments</h2>
<ul>
  {% for post in site.posts %}
    <li>
      <a href="{{ post.url }}">{{ post.title }}</a>
    </li>
  {% endfor %}
</ul>
