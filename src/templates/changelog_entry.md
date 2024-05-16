## [{{ entry.version }}
{% if entry.description %}
{{ entry.description }}
{% endif -%}
{% for commit_type, commits in entry.commits|items %}
### {{ commit_type }}
{% for commit in commits -%}
- {{ commit }}
{% endfor -%}
{% endfor -%}
