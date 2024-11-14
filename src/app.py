import jinja2
import webassets
from fastapi import FastAPI
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
from fastapi.staticfiles import StaticFiles
from fastapi.responses import Response
from fastapi.middleware.gzip import GZipMiddleware


app = FastAPI(response_class=HTMLResponse)
app.add_middleware(GZipMiddleware)
templates = Jinja2Templates(directory="templates")

# Mounts for static content
app.mount("/static", StaticFiles(directory="static"), name="static")
app.mount("/webfonts", StaticFiles(directory="static/webfonts"), name="static")

# CSS Bundling
assets = webassets.Environment()
assets = webassets.Environment(directory='static', url='/static')
bundle_output = 'gen/packed.css'
css = webassets.Bundle(
    'base.css',
    'blog.css',
    'cv.css',
    'navbar.css',
    'portfolio.css',
    'fontawesome/all.min.css',
    filters='cssmin',
    output=bundle_output
)
assets.register('css_all', css)
css.build()

with open(f'static/{bundle_output}') as f:
    BUNDLED_CSS = f.read()

# SSR with jinja2
env = jinja2.Environment(loader=jinja2.FileSystemLoader("templates"))
render = lambda fname: env.get_template(fname).render(css=BUNDLED_CSS)
home_html = render("home.html")
blog_html = render("blog.html")
portfolio_html = render("portfolio.html")
cv_html = render("cv.html")

# Define routes
@app.get("/")
async def home():
    return HTMLResponse(home_html)

@app.get("/blog")
async def blog():
    return HTMLResponse(blog_html)

@app.get("/portfolio")
async def portfolio():
    return HTMLResponse(home_html)

@app.get("/cv")
async def cv():
    return HTMLResponse(cv_html)

@app.get("/health_check")
async def health_check():
    return Response(status_code=200)
