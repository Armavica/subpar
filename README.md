# subpar

Subpar is a filter for paragraph reformatting.

It is similar in purpose to [fmt](https://www.gnu.org/software/coreutils/fmt)
or [par](http://www.nicemice.net/par/), but much simpler and shorter, hence
the name.  In contrast with them however, it is supposed to better handle
non-ascii characters:


``` sh
$ echo "éééé a bbbbbb" | subpar -w9
éééé a
bbbbbb
````

whereas

``` sh
$ echo "éééé a bbbbbb" | par w9
éééé
a bbbbbb
````
