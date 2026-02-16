# Getting Started Tutorial

Build and deploy a simple REST API from scratch using Acme Platform.

## What We'll Build

A simple bookmark API that stores URLs with tags. By the end, you'll have a running service with:

- Three REST endpoints
- PostgreSQL storage
- Health checks
- Deployed to staging

## Step 1: Scaffold the Project

```bash
acme init bookmarks --template=python-fastapi
cd bookmarks
```

## Step 2: Define the Model

Edit `src/models.py`:

```python
from sqlalchemy import Column, String, ARRAY, DateTime
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.sql import func
import uuid

from .database import Base

class Bookmark(Base):
    __tablename__ = "bookmarks"

    id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    url = Column(String(2048), nullable=False)
    title = Column(String(256))
    tags = Column(ARRAY(String), default=[])
    created_at = Column(DateTime(timezone=True), server_default=func.now())
```

## Step 3: Add the Routes

Edit `src/routes/bookmarks.py`:

```python
from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from pydantic import BaseModel
from typing import Optional

router = APIRouter(prefix="/bookmarks", tags=["bookmarks"])

class BookmarkCreate(BaseModel):
    url: str
    title: Optional[str] = None
    tags: list[str] = []

class BookmarkResponse(BaseModel):
    id: str
    url: str
    title: Optional[str]
    tags: list[str]
    created_at: str

@router.get("/")
def list_bookmarks(db: Session = Depends(get_db)):
    return db.query(Bookmark).order_by(Bookmark.created_at.desc()).all()

@router.post("/", status_code=201)
def create_bookmark(data: BookmarkCreate, db: Session = Depends(get_db)):
    bookmark = Bookmark(**data.model_dump())
    db.add(bookmark)
    db.commit()
    db.refresh(bookmark)
    return bookmark

@router.delete("/{bookmark_id}", status_code=204)
def delete_bookmark(bookmark_id: str, db: Session = Depends(get_db)):
    bookmark = db.query(Bookmark).filter(Bookmark.id == bookmark_id).first()
    if not bookmark:
        raise HTTPException(status_code=404, detail="Bookmark not found")
    db.delete(bookmark)
    db.commit()
```

## Step 4: Deploy

```bash
acme deploy --env staging
```

```
Building image... done (12s)
Pushing to registry... done (4s)
Deploying to staging... done (18s)

Service:  bookmarks
URL:      https://bookmarks.staging.acme.internal
Status:   healthy
```

## Step 5: Test It

```bash
# create a bookmark
curl -X POST https://bookmarks.staging.acme.internal/bookmarks \
  -H "Content-Type: application/json" \
  -d '{"url": "https://rust-lang.org", "title": "Rust", "tags": ["lang", "systems"]}'

# list bookmarks
curl https://bookmarks.staging.acme.internal/bookmarks
```

## Next Steps

- Add [authentication](../api/authentication.md) to protect your endpoints
- Set up [monitoring](monitoring.md) dashboards
- Configure [auto-scaling](../guides/configuration.md) for production
