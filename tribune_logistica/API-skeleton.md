# Book Legion API - Skeleton

This document outlines the API endpoints for the Book Legion system. It is designed to interact with the backend components already implemented, including:

- User authentication & authorization
- Book and manifest storage
- Epub processing & chunked audiobook streaming
- Per-user cursors

---

## 1. Authentication Endpoints

### 1.1 Register User
**POST** `/api/v1/register`   //works

**Request Body**

```
{
  "username": "pete",
  "password": "secret123"
}
```

**Response**
```
{
  "success": true,
  "message": "User created"
}
```

#### Handler Responsibilities

Hash password with Argon2id

Insert user into storage

Return success/failure


### 1.2 Login User
POST /api/v1/login   //works

**Request Body**

```
{
  "username": "pete",
  "password": "secret123"
}
```
**Response**
```
{
  "auth_token": "jwt_or_random_token",
  "refresh_token": "refresh_token_value",
  "expires_in": 900
}
```


#### Handler Responsibilities

Verify password

Generate refresh token & auth token

Set TTL for auth token

Return tokens and expiry

### 1.3 Refresh Auth Token
POST /api/v1/refreshtoken //works

**Request Body**


```
{
  "refresh_token": "refresh_token_value"
}
```

**Response**



```
{
  "auth_token": "new_auth_token",
  "expires_in": 900
}
```
#### Handler Responsibilities

Validate refresh token

Generate new auth token

Return token & TTL

## 2. Library Endpoints

### 2.1 Get Single Book
GET /api/v1/books/{book_id} // works

**Response**



```
{
  "id": "b1",
  "title": "Book One",
  "author_id": "a1",
  "series_id": "s1",
  "series_order": 1,
  "file_path": "/path/to/book.epub"
}
```
#### Handler Responsibilities

Load book by ID

Return 404 if not found

### 2.2 Get Library Manifest
GET /api/v1/manifest   //works

**Response**



```
{
  "series": [
    {
      "series_id": "s1",
      "first_book_id": "b1"
    }
  ]
}
```
#### Handler Responsibilities

Load manifest from storage

Return all series entries

## 3. Book Reading / Audiobook Streaming
### 3.0 Request Cursor
GET /api/v1/cursors/{book_id}   //works.
**Response**
```
{
  "UserID": "u1",
  "BookID": "b1",
  "Cursor":{
    "Chapter": 1
    "Chunk": 1
  }
}
```
#### Handler Responsibilities
Load cursor for a specific book.
Creates and returns a new cursor for the book, if one does not exists


### 3.1 Get Chapter
GET /api/v1/books/{book_id}/chapters/{chapter_index} // works

**Response**
```
{
  "chapter_index": 0,
  "num_chunks": 5,
  "text": xHTML
}
```
#### Handler Responsibilities

Load epub

Extract chapter and number of chunks

### 3.2 Get Chunks
GET /api/v1/books/{book_id}/chunks // does not work, needs further investigation
```
{
  "UserCursor":{
    "BookID": "b1",
    "UserID": u1,
    "Cursor":{
      "Chapter": 1,
      "Chunk": 2
    }
  }
  "requestSize": 10
}
```
**Response**

```
[
  {
    "data": "..."
    "Cursor": {
      "Chapter": 1,
      "Chunk": 2
    }
  }
]
```


### 3.3 Get Nav:
GET /api/v1/books/{book_id}/nav // works

**Response**
```
[
  "PrettySpineItem": {
	"Index":  0
	"Number": 1
	"Title": "title"
  }
]

```


#### Handler Responsibilities

Retrieve specific chunks using buffer

Validate against chapter boundaries

Serve chunk as raw bytes or base64-encoded

## 5. Miscellaneous Endpoints
### 5.1 Get Cover Image
GET /api/v1/books/{book_id}/cover  //does not work?

**Response**

image/jpeg or image/png binary stream

#### Handler Responsibilities

Extract cover from epub

Return binary stream directly

### 5.2 Get CSS Files
GET /api/v1/books/{book_id}/css   //works

**Response**

CSS text

#### Handler Responsibilities

Extract all CSS files from epub

Return as list of filename + content



### 5.3 Save Cursors
POST /api/v1/cursors/save // works
```
{
  "UserID": "u1",
  "BookID": "b1",
  "Cursor":{
    "Chapter": 1
    "Chunk": 1
  }
}
```
**Response**: 200/404


### 5.4 Save Book
POST /api/v1/savebook   //works
```
{
  "id": "b1",
  "title": "Book One",
  "author_id": "a1",
  "series_id": "s1",
  "series_order": 1,
  "file_path": "/path/to/book.epub"
}
```
**Response**: 200/500

#### Handler Responsibilities
Save the posted cursor


## 6. Notes
All endpoints require auth token except registration and login and cover

User-specific endpoints (cursor, next chunks) validate that token belongs to user

All file-related operations (ValidateBooks, chunks, cover) must ensure the file exists

Manifest and series endpoints are read-only

This skeleton maps directly to your existing system, so the handlers can reuse:

library.LoadBook / SaveBook

library.LoadManifest / SaveManifest

cursor buffer logic

login auth tokens and user session management

epub extraction utilities

You now have everything needed to implement a full REST API that serves books, audiobooks, and manages users.

pgsql
