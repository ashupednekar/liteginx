from fastapi import FastAPI

app = FastAPI()

@app.get("/one")
def get_one():
    return {"message": "one"}

@app.get("/")
def get_root():
    return {"message": "two"}
