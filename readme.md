Work Tock
==========

A command line work tracking program.

See Documentation for basic file format

otherwise the --help command should be useful enough.



Basic Usage
----------

You can use this program to clockin

    work_tock -i <JobName>

clockout

    work_tock -o

Or print a record of recent clock entries with flags

    work_tock --job_s dothing -p --since 03/04/2020


The program works with a single text file that is easy to edit if needed. The program will never overwrite your file, only read and append, so all data in that file will otherwise remain untouched.

To set the location of the core file, the default config for your program can be found in "$HOME/.config/work\_tock/init.toml 

```toml
[config]
    # Set path the the current working file 
    # anything within "{}" is read as an environment variable
    file="{HOME}/<path>/<to>/<file>"

    #Optional require all job entries to be snake_case
    snake=true  
    
    #camel=true  #if you prefer camelCase
```

A standard file looks like this :

```toml
$home_jobs[car_wash,eat]
23/01/2019
    car_wash,12:30-13:50
    15:00,#car_wash is implied by previous Job
    programming,16:00,#Clockout for car_wash is implied by new Job
    eat,17:00
  -18:00,#Clockout

24/01/2019
    12:00,#Eating is implied as it was the last job
  -13:00
    programming,14:00
  -16:00
```

* Commas and newlines begin a new item
* Whitespace is ignored next to either side or a break (",","\n")
* Jobs are Letters only
* Clockins are "hh:mm"
* Clockouts are  "-hh:mm"
* Tags begin with an "\_" and can be cleared with "\_\_"
* Dates are dd/mm/yyyy, but if you set year=2019, dates can be dd/mm after that.
* Groups are defined by a ```$group_name[list,of,jobs,in,group]```

Every Clockin will use the most recent Job,Date, and Tags for the clocking, 

So given the example file ```work_tock``` will produce:

```toml
{"car_wash": 02:20, "eat": 04:00, "programming": 01:00}

Total Time = 07:20

```

Printing and Filters
------------------

Using "-p" Will print all entries, but if you want to be morse spcific you can apply a filter and -p will print only entries that pass that filter.

> NOTE: -t for today used to be -d which is now used to specify a date to work on

To get more relevent data you can use filters such as "-t" :Today, or "--day 3/1/2019", or by job 

eg: ```work_tock -p --job car_wash``` will return

```toml
23/01/2019
  car_wash: 12:30-13:50 = 01:20   => 01:20
  car_wash: 15:00-16:00 = 01:00   => 02:20

{"car_wash": 02:20}

Total Time = 02:20

```

or ```work_tock -p --group home_jobs``` will produce:

```toml
Filtering by group home_jobs
23/01/2019
  car_wash: 12:30-13:50 = 01:20   => 01:20
  car_wash: 15:00-16:00 = 01:00   => 02:20
  eat: 17:00-18:00 = 01:00   => 03:20
24/01/2019
  eat: 12:00-13:00 = 01:00   => 04:20

{"car_wash": 02:20, "eat": 02:00}

Total Time = 04:20

```




For more information use ```work_tock --help```





changes:
=========

## v 0.2.0

* Now has job groups allowing you to define a group of jobs
* Now allows you to require camel or snake case in the config file

## v 0.1.8

* --outat no longer exists instead use -o -a <timeout>
* --in now use -a (at time) and -d (on date) to set date and time instead of comma separated parsing.

to help with issues arising from forgetting to clockout yesterday, -y can be used to treat the current day as yesterday, eg: yesterday I clocked in at 12:00 and forgot to clockout at 17:00

    work_tock -oy -a 17:00 

means

    work_tock out yesterday at 17:00



v 0.1.7
--------

Bug fix -- Now handles empty files properly (Oops)

now takes -q option for clockin to previous Job

v 0.1.6
---------

Added error handling for loggin in and out on different days.

switched order of logging in and printed statements to include the current logout on the printed statements.


v 0.1.5
-----------

Documented use of toml instead of lazyf

v 0.1.4
--------
Separated library from application

v 0.1.3
-------
Trying to get repository and docs showing on Cargo

v 0.1.2
--------
Added Docs to readme


v 0.1.1
--------

Added basic usage documentation

