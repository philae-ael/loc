#!/bin/sh
gen ()
{
    printf "$(cat ./Readme-head.md)\n$(cargo run . 2>/dev/null)$(cat ./Readme-tail.md)" > ./Readme.md
}

while :
do
    gen
    cur=$(sha256sum ./Readme.md)
    echo $cur $last

    if [[ "$cur" == "$last" ]] then
		    break
    fi

    last=$cur
done

