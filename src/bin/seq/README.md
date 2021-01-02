crab seq
========

Name
----

seq -- print sequences of numbers

Synopsis
--------

    seq [-w] [-s string] [-t string] [first [incr]] last

Description
-----------

The seq utility prints a sequence of numbers, one per line (default), from first
(default 1), to near last as possible, in increments of incr (default 1). When
first is larger than last the default incr is -1.

All numbers are interpreted as floating point.

Normally integer values are printed as decimal integers.

The seq utility accepts the following options:

> ### `-s	string`
> 
> Use string to separate numbers. The default is \n.
> 
> ### `-t	string`
> 
> Use string to terminate sequence of numbers. This option is useful when the
> separator does not contain a newline.
> 
> ### `-w`
> 
> Equalize the widths of all numbers by padding with zeros as necessary. 

The seq utility exits 0 onsuccess and non-zero if an error occurs.

Examples
--------

    # seq 1 3
	1
	2
	3

    # seq 3 1
    3
    2
    1

    seq -w 0 .05 .1
    0.00
	0.05
	0.10

History
-------

The seq command first appeared in Plan 9 from Bell Labs. A seq command appeared
in NetBSD 3.0, and ported to FreeBSD 9.0. This command was based on the command
of the same name in FreeBSD, which itself was based on the seq command in Plan 9
from Bell Labs and the GNU core utilities.

Compatibility
-------------

This implementation of the seq command varies from the GNU and FreeBSD variants
as follows:

*   Negative numbers are only supported after all options and a bare `--`.
    Otherwise they confuse the option parser. For example, this invocation fails:

    ```
    # seq -1
    Unrecognized option: '1'
    ```

    But this succeeds:

    ```
    seq -- -1
    1
    0
    -1
    ```

*   There is no `-f format` option, mainly because Rust does not have a `printf`
    interface.

*   Decimal formatting is always displayed at the highest precision of all
    inputs padded with zeros, unlike other implementations, which only show the
    full precision for every number when `-w` is set (and crab seq is fully
    compatible with `-w`). This table demonstrates the differences:

    ```
     FreeBSD seq     crab seq         FreeBSD with -w     crab seq with -w
    ---------------+----------------+-------------------+-------------------
    # seq 9 .25 10 | # seq 9 .25 10 | # seq -w 9 .25 10 | # seq -w 9 .25 10
    9              | 9.00           | 09.00             | 09.00
    9.25           | 9.25           | 09.25             | 09.25
    9.5            | 9.50           | 09.50             | 09.50
    9.75           | 9.75           | 09.75             | 09.75
    10             | 10.00          | 10.00             | 10.00
    ```
