from fastapi import FastAPI, Request
from fastapi.templating import Jinja2Templates
from fastapi.staticfiles import StaticFiles
from fastapi.responses import Response

app = FastAPI()
templates = Jinja2Templates(directory="templates")
app.mount("/static", StaticFiles(directory="static"), name="static")

@app.get("/")
async def home(request: Request):
    return templates.TemplateResponse("home.html", dict(request=request))

@app.get("/blog")
async def blog(request: Request):
    return templates.TemplateResponse("blog.html", dict(request=request))

@app.get("/portfolio")
async def portfolio(request: Request):
    return templates.TemplateResponse("portfolio.html", dict(request=request))

@app.get("/cv")
async def cv(request: Request):
    return templates.TemplateResponse("cv.html", dict(request=request))

@app.get("/health_check")
async def health_check():
    return Response(status_code=200)
