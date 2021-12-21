# CDN
FerrisChat's CDN, running on `cdn.ferris.chat`. Serves uploaded images and other files.

## Setup Instructions
**In Redis**:

`SET max_content_length {max_content_length in bytes}`

`HSET cdn_nodes {node_id e.g. 1} {node_ip e.g. 127.0.0.1}`

**Fill in the following vars in the .env file:**

`FC_CDN_HOST={the hostname of the cdn e.g. https://cdn.ferris.chat}`

`FC_CDN_UPLOADS_PATH={the path to the uploads directory e.g. ../../uploads}`

`FC_CDN_REDIS_URL={the url of the redis instance e.g. redis://192.168.1.2:6379}`

`FC_CDN_CACHE={Whether to cache the files or not, defaults to true}`

`FC_CDN_CACHE_SIZE={the max size of the cache in bytes e.g. 100000000 defaults to system's memory * 0.25}`
