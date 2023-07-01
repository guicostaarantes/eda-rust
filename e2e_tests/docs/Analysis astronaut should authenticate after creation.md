## Astronaut should authenticate after creation

### Run 1

Parameters:
    - Replicas for astronauts-msvc:              3
    - CPU limit for astronauts-msvc.service:     250m
    - Memory limit for astronauts-msvc.service:  200Mi
    - CPU limit for astronauts-msvc.mongo:       250m
    - Memory limit for astronauts-msvc.mongo:    200Mi
    - Replicas for auth-msvc:                    3
    - CPU limit for auth-msvc.service:           250m
    - Memory limit for auth-msvc.service:        200Mi
    - CPU limit for auth-msvc.mongo:             250m
    - Memory limit for auth-msvc.mongo:          200Mi
    - Iterations:                                10
    - Concurrent operations:                     200
    - Sleep between creation and authentication: 1000ms

Results:
    - Finish time:                               257.83s

### Run 2

Parameters:
    - Same as run 1, except:
    - CPU limit for astronauts-msvc.service:     500m
    - CPU limit for auth-msvc.service:           500m

Results:
    - Finish time:                               126.52s 

### Run 3

Parameters:
    - Same as run 1, except:
    - CPU limit for astronauts-msvc.service:     1000m
    - CPU limit for auth-msvc.service:           1000m

Results:
    - Finish time:                               76.47s 

### Run 4

Parameters:
    - Same as run 2, except:
    - Replicas for astronauts-msvc:              2
    - Replicas for auth-msvc:                    2

Results:
    - Finish time:                               174.16s

### Run 5

Parameters:
    - Same as run 2, except:
    - Replicas for astronauts-msvc:              6
    - Replicas for auth-msvc:                    6

Results:
    - Finish time:                               113.45s

### Conclusions

For the specified load of 200 concurrent users, the usage of 3 replicas had a great impact on performance compared to 2 replicas. Using 6 replicas instead of 3 showed a small performance impact.

Increasing the CPU limits has showed performance impact both from 250m to 500m and 500m to 1000m. Hashing passwords is a CPU-intensive operation so this makes sense.
