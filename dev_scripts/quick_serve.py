#!/bin/env python3
"""
Quick script to test html/css changes without waiting for rust/docker build to complete.
"""
import atexit
import os
from pathlib import Path
import os
import shutil
import http.server
import signal
import sys

import jinja2
import webassets

def remove_file(path):
    # make sure we're working from root
    print("Cleaning up", path)
    if os.path.isfile(path):
        os.remove(path)

def remove_dir(dir):
    # make sure we're working from root
    print("Cleaning up", dir)
    shutil.rmtree(dir, ignore_errors=False)

def sigint_handler(_signum, _fname):
    print("\nSIGINT Received. Exiting...", file=sys.stderr,)
    sys.exit(0)

signal.signal(signal.SIGINT, sigint_handler)

if __name__ == "__main__":
    # Work in static_site root
    os.chdir(Path(__file__).parent.parent)
    
    # Temp files
    _TEMP_BUNDLE = Path("_bundle.css").resolve()
    _TEMP_STATIC_DIR = Path("_static").resolve()

    atexit.register(remove_file, _TEMP_BUNDLE)
    atexit.register(remove_dir, _TEMP_STATIC_DIR)

    # Perform CSS Bundling
    assets = webassets.Environment()
    assets = webassets.Environment(directory='styles', url='/styles')
    css = webassets.Bundle(
        'base.css',
        'blog.css',
        'cv.css',
        'navbar.css',
        'portfolio.css',
        filters='cssmin',
        output=str(_TEMP_BUNDLE)
    )
    assets.register('css_all', css)
    css.build()
    
    with open(_TEMP_BUNDLE) as f:
        BUNDLED_CSS = f.read()
    
    # SSR with jinja2 and leave generated static content in build directory
    env = jinja2.Environment(loader=jinja2.FileSystemLoader("templates"))
    render = lambda fname: env.get_template(fname).render(css=BUNDLED_CSS)

    os.makedirs(_TEMP_STATIC_DIR)
    for fname in ["index.html", "blog.html", "portfolio.html", "cv.html"]:
        # Strip html from filename as hack - extension not included in links
        with open(_TEMP_STATIC_DIR / fname.strip(".html"), 'w') as f:
            f.write(render(fname))

    os.chdir(_TEMP_STATIC_DIR)
    port = 8000
    # Default to html content type for endpoints - they don't include .html
    # extensions
    http.server.SimpleHTTPRequestHandler.extensions_map[""] = "text/html"
    httpd = http.server.HTTPServer(('', port), http.server.SimpleHTTPRequestHandler)
    print("Serving on port", port)
    httpd.serve_forever()
    
