from fastapi import FastAPI, Request
from fastapi.templating import Jinja2Templates
from fastapi.staticfiles import StaticFiles
from fastapi.responses import Response
from fastapi.middleware.gzip import GZipMiddleware
from webassets import Environment, Bundle


app = FastAPI()
app.add_middleware(GZipMiddleware)
templates = Jinja2Templates(directory="templates")

# Mounts for static content
app.mount("/static", StaticFiles(directory="static"), name="static")
app.mount("/webfonts", StaticFiles(directory="static/webfonts"), name="static")

assets = Environment()
assets = Environment(directory='static', url='/static')
bundle_output = 'gen/packed.css'
css = Bundle(
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

@app.get("/")
async def home(request: Request):
    return templates.TemplateResponse("home.html", dict(request=request, css=BUNDLED_CSS))

@app.get("/blog")
async def blog(request: Request):
    return templates.TemplateResponse("blog.html", dict(request=request, css=BUNDLED_CSS))

@app.get("/portfolio")
async def portfolio(request: Request):
    return templates.TemplateResponse("portfolio.html", dict(request=request, css=BUNDLED_CSS))

@app.get("/cv")
async def cv(request: Request):
    return templates.TemplateResponse("cv.html", dict(request=request, css=BUNDLED_CSS))

@app.get("/health_check")
async def health_check():
    return Response(status_code=200)
