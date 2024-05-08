## {{ entry.version }}
{% for commit_type, commits in entry.commits|items -%}
#### {{ commit_type }}
{% for commit in commits -%}
- {{ commit.summary }} - ({{ commit.hash }}) - {{ commit.author }}
{% endfor -%}
{% endfor -%}
