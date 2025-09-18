# Personal Typst Browser

Browse your personal typst files. Written in Rust with axum.

You can preview or download every file you want. Typst files are compiled on the fly and sent over as a pdf.

Example of the root page:

<img width="749" height="652" alt="image" src="https://github.com/user-attachments/assets/b3f78b0c-8b35-4499-bebf-3ad393a2b21b" />

Note that nothing is cached, meaning it could take up to a few seconds to compile and download a typst file.
But most of the times the request will complete in under a second.

## Deploy

The easiest way to deploy this is with Docker Compose.

You first have to build the docker image. To do that, clone this repository and change into the directory, then run:

```
docker build --tag personal_typst_browser .
```

This builds an image with the name `personal_typst_browser`. No dependencies other than Docker are needed.

The best way to serve this image is with Docker Compose:

```yml
services:
  typst-browser:
    image: personal_typst_browser
    ports:
      - 80:3000 # Choose whatever port you want
    volumes:
      - /local/path/to/files:/files # Example mount for the files that are to be served.
      - ./.env:/app/.env
```

Example `.env` file:

```env
# root directory of the typst content
ROOT_DIR=/files

# Comma seperated values of ignored files
EXCLUDE_FILES=.DS_Store,.git

# Prefix of the URL this is ultimately served on
URL_PREFIX=http://example.com

# Host and port. Set to 0.0.0.0:3000 by default
HOST=0.0.0.0:3000
```

## License

This code is licensed under [MIT](/LICENSE)
