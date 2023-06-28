#!bin/bash

file_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )

msvcs=( "auth-msvc" "astronauts-msvc" "missions-msvc" )

cd $file_path
kubectl delete -k dev

x=0;
while [ $x != ${#msvcs[@]} ]
do
	docker build -t k8s-${msvcs[$x]} ../${msvcs[$x]}
	let "x = x +1"
done

kubectl apply -k dev
