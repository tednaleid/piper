# todo

- get stdin working again

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



