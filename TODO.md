# todo

- refactor the input reader so that it is a nom parser that can split records/fields on whatever the user has told us to split on
- we eventually want to delete the context module, or totally refactor it to use the parser

on startup, we parse any template arguments with the nom parser and create a request_context object out of it

request_context has

mvp:
    X resolved url
    X http method  
    X ordinal request number
next:
    - resolved request body
    - resolved request headers
    - input record fields?
        - in case we want to carry context forward
        - this can start dumb and always be present, we could remove it later if we don't care about it because it isn't in an output template


request_builder needs
mvp:
    - url template
      - `-u/--url <template>`
      - ex: "http://example.com/widget?id={1}"
    X method literal
next:  
    - POST body template
      - `-b/--body <template>`
      - ex: "{1}"
    - headers templates  
      - `-H/--header <template>`
      - multiple allowed? or split on something like `:` and kv pairs?
      - takes care of the mime type/accept/etc

    - insecure https TLS verification? (skip checking certs)
    - throttle/workers
    - method template?


  
response_output can have
mvp:
    - just output body text as a string
next:
    - response template
    - response headers as template
        - `{%header_name%}`?
            - would that just be the value then?
            - null when not present? or empty string
        - or `%H` to emit all headers
            - as n=v CSV?
            - or just in the JSON output?
    - request/response timing duration
    - request timestamp 
        - ISO-8601
        - or unix timestamp
    - response payload length
    - request url
    - response status code
    - standard json output
    - other output types
        - uuencoded
        - url/json escaped
        - hash of bytes
    - config for output (stdout/files)
    - per-request results on stderr (or quiet)
    - colorized or not
    - JSON envelope
        - could auto include "everything"
            - how would the context stuff work here, maybe just a single "-C" context string



# todo post requwest/tokio 1.0 update

- can we abstract away the actual request so that we can swap out the back end call with a passed in function?
- have a dry run that emits the (curl?) expression


# Running/Testing

run the echoserver:

    cargo run --release --package echoserver
    
This can be tested with:

    curl -sL "http://localhost:7878/ping"
    pong 


run the app to hit that server:

    seq 1000000 | awk '{printf "http://localhost:7878/ping\n", $1}' | cargo run --release -- | pv -albert > /dev/null
    
    
    seq 10 | cargo run --release -- -u "http://localhost:7878/ping?id={1}"
    seq 10 | cargo run -- -u "http://localhost:7878/ping?id={1}"


ganda comparison:

seq 1000000 | awk '{printf "http://localhost:8000/foobar/%s\n", $1}' | ganda -W 30 2>&1 | pv -albert > /dev/null
^C62k 0:00:08 [44.3k/s] [45.3k/s]

