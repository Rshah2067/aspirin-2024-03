I found that having around 25 threads resulted in the fastest runtime of about 2.977 seconds for 10million elements. I did this very unscientifically using std::time as I couldn't figure out how to use critereion, but I tried to run around 5 tests for 100,75,50,25,10, 2,and 1 threads. The following is the average times I saw:
100 threads 3.21 seconds 
75 threads 3.002 seconds
50 threads 2.9461
25 2.92 seconds
10 2.977
2 3.2 seconds
1 3.4957 3.529
The big benifit of having more threads is that in any downtime we can keep working, however past a certain amount of threads, I believe there just isn't any more downtime to exploit. Creating threads itself, has overhead, and having to merge the results from more threads adds some cost.