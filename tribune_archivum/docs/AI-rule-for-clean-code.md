Write code that can be understood with a short read, without comments. If you feel the need to comment code inside a function, extract that code into a separate function named so that its purpose is clear from the name. Only extract when a short read isn't enough.

Doc comments above a function are allowed when the how or why can't be conveyed in its name. Keep them to reasoning a reader can't recover from the code.

A function must earn its keep: the call site has to read more easily than the code it replaced. A function that only wraps a single expression, or that is called from one place and is no clearer than its body, usually doesn't.