# todo
- exit when EOF/end of input
- split out requests from awaiting responses?

- make lots of workers?
  - work stealing thread pool 
      - https://docs.rs/tokio-threadpool/0.1.17/tokio_threadpool/struct.ThreadPool.html  ?
  - tokio task?
    - https://tokio-rs.github.io/tokio/doc/tokio/task/index.html
- throttling?


- tests with tokio runtime?



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



# Misc
have a channel (crossbeam?) of requests to make

using async-std we could "spawn" on each request to make

inside that spawn we will `await` the result, then send that down another mpsc



# Look at

- work stealing queues?
- making a future resolver that iterates over the waiting futures and resolves anything that's ready
- look at examples of rust based load testing tools


# Options

# single worker
- stream of urls/requests that are picked up and sent by a single worker which spawns a future on a runtime

inside the future the request is made
- how does the client persist the connection in this mode?

future has an and_then that sends the results into another mpsc queue where it is printed out

what awaits that future? does it automatically get run

concurrency happens because the scheduler is spawning/resolving the tasks

throttling could be done by how many get sent to the queue every second via a tick, or an `after`
if we used an `after` we could check the time before we gather the N things, then when we've sent the N things 
see if we need to delay some, if we do, schedule an `after` with the necessary remaining delay

# N workers

each listening to an mpmc crossbeam channel

when each gets an url it makes the request and then awaits it right there


looks promising:

https://github.com/alanbondarun/butterfly/blob/master/src/main.rs  

with delay/throttling:

https://github.com/felipe-fg/http-storm/blob/master/src/worker.rs

also FuturesUnordered: 

http://jamesmcm.github.io/blog/2020/05/06/a-practical-introduction-to-async-programming-in-rust/#en




