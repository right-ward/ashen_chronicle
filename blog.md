---
layout: default
title: Blog
permalink: /blog/
description: "Posts and notes by right-ward."
---

# Blog

<p>Read posts here or subscribe using the <a href="{{ '/feed.xml' | relative_url }}">Atom feed</a>.</p>

{% assign blog_posts = site.posts %}
{% if blog_posts.size > 0 %}
<ul>
{% for post in blog_posts %}
  <li>
    <time datetime="{{ post.date | date_to_xmlschema }}">{{ post.date | date: "%B %-d, %Y" }}</time> — <a href="{{ post.url | relative_url }}">{{ post.title | escape }}</a>
    {% if post.excerpt %}
    <p>{{ post.excerpt | strip_html | normalize_whitespace | truncate: 240 | escape }}</p>
    {% endif %}
  </li>
{% endfor %}
</ul>
{% else %}
<p>No posts published yet. New posts will appear here automatically once they are added to <code>_posts/</code>.</p>
{% endif %}
