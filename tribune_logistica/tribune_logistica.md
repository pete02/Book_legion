# Tribune Logistica
- Client can start by intializing the book, by asking for current position in a book. 
    - If no book is initialized, the chunks are not returned. Instead error is returned, instructing that no book is initialized
    - Closing a websocket will count as de-initializing the book, upon which the current place is saved from memory to disk, and all files are closed.
    - Reconnection /break in websocket will not count as de-initializing the book, if the client connects within 30 sec.

- Above to be implemented via websocket connection.

- The API can also be queried for all locations each book, calculated as a presentage, with out initializing the book
    - the presentage is saved as current chunk out of all chunks.
    - querying all the locations does not initialize any books. This will be pure https request.

### Completed, but not hooked up to web socket:
- can fetch library manifest to string
- Can initialize a book as described
- Can update the progress into the library manifest
- Can fetch a certain chapter as epub
- Can fetch a chunk from an audiobook, from current page and chunk to the given page and chunk. However, nothing is stored in memory



### Endpoints:
- BookStatus:{
    "name",
    "path",
    "page",
    "chunk",
    "json"
}


- /init?name=book,type=book_type
    will initialize a book
    Returns BookStatus struct
    tested, ok

- POST /book
    will return the current page, based on the BookStatus struct
    Updates the progress to match current page
    Requires BookStatus struct
    tested, ok

- POST /audio?chunk=x
    will return the the the audio between the current chunk and the desired chunk, as well as a flag whether or not we reached end of the page. Requires BookStatus struct
    tested, ok

- POST /audiomap
    Will return the audio to chunk map 
    Requires BookStatus struct
    tested, ok

- POST /update
    Saves the given BookStatus struct to the library manifest
    Requires BookStatus struct
    tested, ok

- /manifest
    returns the library manifest, holding information about the contents of the library
    tested, ok

- /cover/{book}
    returns a jpg of the cover of the given book, if it is found
    tested, ok
- /css/{book}
    returns css of a given book, if found
    tested, ok

### Security:
- All API usage will require a separate webtoken. This API cannot generate, nor allow new web tokens.
- API reads the token from Authorization.json, and compares the incoming webtoken for that. If the token is not correct, all actions are to be refused

### Data:
    library folder that holds a folder for each book, conisting of:
        - mp3
        - epub
        - chunkmap-json. 
    - Location.json:
        - for each book:
            - location in the library folder
            - current chunk
            - max chunks
    - Authorization.json
        Holds the current webtoken, that has access to the service

