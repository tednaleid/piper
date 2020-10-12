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

    
# argument parsing/context

use cases:

  - the body bytes from the output
    - the primary thing we want
    - could be a string, or binary
    - could want it to be encoded/escaped
  - values on the input (context ids) that don't actually get sent to the server or come back on the response
    - we want it on the output to help us correlate
  - 
  
Things we want to be able to format and send to the server:
- the url
    - -u/--url <format string>
- a body string/binary
    -b / --body <format string>
- the HTTP method
    -- -X/method <string>
- 1..N headers with key/values
    -- -H/--header <format string> -- is this a single string that we break down, split on `:`, kv pairs?
    - some could be static, some could be per request
    - includes the mime type
  
Things we definitely want the ability to output:
  - the return HTTP code
    - `%X`
  - the response body/binary
    - `%s` 
  - the request url
    - `%u`
  - input context values (correlation IDs)
    - `%1`..`%N` or format string input, same as pipem?
  
Possible things to output:
  - response headers
    - `%H{name}`? 
      - would that just be the value then?
    - or `%H` to emit all headers
      - as n=v CSV?
    - or just in the JSON output?
  - response payload length
  - a hash of the response
  - a request timestamp
    - unix epoch or ISO-8601
  - the request body
  - request payload length
  - request timing data
  - actual host for redirects?
  - different output delimiters than \n
  
  
  - JSON envelope
    - could auto include "everything"
      - how would the context stuff work here, maybe just a single "-C" context string
  
  - a file/directory to write output to?


possible arguments:

examples:

    --context <format string>, -c <format string> - the context for the request  
      - how would this be used on the output

# todo

- errors should error

- tests

have request_context flow through so we can retry

allow incoming tsv with "$1 $2" style templates
or json with "${.name}" style templates (json path)

have a dry run that emits the (curl?) expression



- fix panics if broken pipe
- send responses (with body?) down a channel

input line -> request_context -> make request -> ack_request 
                                          \_> finish request


ganda comparison:

seq 1000000 | awk '{printf "http://localhost:8000/foobar/%s\n", $1}' | ganda -W 30 2>&1 | pv -albert > /dev/null
^C62k 0:00:08 [44.3k/s] [45.3k/s]

# organization

1. create request contexts
has:
- url
- headers
- body
- other context to carry along


to instantiate
- request template
    - url template?
    - headers template?
    - body template?
    - context template?

2. make requests


3. parse responses

 
    // create requests

    // make requests
        // request 

    // response
    // stats?



