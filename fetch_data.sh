#! /usr/bin/env bash

echo "Fetching buses"
wget -q 'https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/Bus!A1:Q100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak' -O data/buses.json

echo "Fetching schedule"
wget -q 'https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/BusOperate!A1:Q100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak' -O data/schedule.json

echo "Fetching stops"
wget -q 'https://sheets.googleapis.com/v4/spreadsheets/1lj9lfPBxlHo_5eSlm-APASlEWUqzCiccGQDlVlAM9SE/values/BusStop!A1:100/?key=AIzaSyCoS3cw1N9C2pY-WUXRnAAPC5N3sKdd_ak' -O data/stops.json
