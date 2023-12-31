#!bin/bash

file_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

msvcs=( "auth-msvc" "astronauts-msvc" "missions-msvc" )

cd $file_path

# kubectl delete -k dev

x=0;
while [ $x != ${#msvcs[@]} ]
do
	docker build -t ${msvcs[$x]} ../${msvcs[$x]}
	docker tag ${msvcs[$x]} localhost:5001/${msvcs[$x]}
	docker push localhost:5001/${msvcs[$x]}
	docker rmi localhost:5001/${msvcs[$x]}
	let "x = x + 1"
done

# kubectl apply -k dev
