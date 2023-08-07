#!/usr/bin/env perl

my $error = "";
while(my $l = <>) {
    if($l =~ m/^(error|warning)(\[\w+\])?:\s*(.+)/) {
        $error = $3;
    } elsif($l =~ m/^\s+-->\s*([\w\.]+):(\d+):(\d+)/) {
        print("$1:$2:$3: $error\n");
    } else {
        print $l;
    }
}
