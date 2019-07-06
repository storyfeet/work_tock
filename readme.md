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


The program works with a single text file, and will only read, and append to it, so all data in that file will otherwise remain untouched.

use "-f" to select the file or set it in "$HOME/.config/work\_tock/init"

```
config:
    file:{HOME}/<path>/<to>/<file>
```

"{HOME}" is the env var $HOME, Any other env var can be used or not.

A standard file looks like this :

```
23/01/2019
    Carwashing,12:30-13:50
    15:00,#Carwashing is implied by previous Job
    Programming,16:00,#Clockout for Carwash is implied by new Job
    Eating,17:00
  -18:00,#Clockout

24/01/2019
    _breakfast,#Tags can be added with underscore
    15:00,#Eating is implied as it was the last job
    __,#clears all current tags.
  -16:00
```

* Commas and newlines begin a new item
* Whitespace is ignored next to either side or a break (",","\n")
* Jobs are Letters only
* Clockins are "hh:mm"
* Clockouts are  "-hh:mm"
* Tags begin with an "\_" and can be cleared with "\_\_"
* Dates are dd/mm/yyyy, but if you set year=2019, dates can be dd/mm after that.

Every Clockin will use the most recent Job,Date, and Tags for the clocking, 

So given the example file ```work_tock``` will produce:

```
{"Carwashing": 02:20, "Eating": 02:00, "Programming": 01:00}

Total Time = 05:20
```

Printing and Filters
------------------

Using "-p" Will print all entries, but if you want to be morse spcific you can apply a filter and -p will print only entries that pass that filter.

To get more relevent data you can use filters such as "-d" :Today, or "--day 3/1/2019", or by job 

eg: ```work_tock -p --job Carwashing``` will return

```
23/01/2019
  Carwashing: 12:30-13:50 = 01:20   => 01:20
  Carwashing: 15:00-16:00 = 01:00   => 02:20

{"Carwashing": 02:20}

Total Time = 02:20
```

For more information use ```work_tock --help```





changes:
=========

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

