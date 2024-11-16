#!/bin/env python3
import os
from pathlib import Path

import jinja2
import webassets

# Work in project root
os.chdir(Path(__file__).parent.parent)

# Perform CSS Bundling
assets = webassets.Environment()
assets = webassets.Environment(directory='styles', url='/styles')
bundle_output = 'gen/packed.css'
css = webassets.Bundle(
    'base.css',
    'blog.css',
    'cv.css',
    'navbar.css',
    'portfolio.css',
    #'fontawesome/all.min.css',
    filters='cssmin',
    output=bundle_output
)
assets.register('css_all', css)
css.build()

with open(f'styles/{bundle_output}') as f:
    BUNDLED_CSS = f.read()

# SSR with jinja2 and leave static files in static directory
env = jinja2.Environment(loader=jinja2.FileSystemLoader("templates"))
render = lambda fname: env.get_template(fname).render(css=BUNDLED_CSS)

os.makedirs('static', exist_ok=True)
for fname in ["index.html", "blog.html", "portfolio.html", "cv.html"]:
    with open(f'static/{fname}', 'w') as f:
        f.write(render(fname))
