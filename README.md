# Instructions

1. export $(cat .env | xargs)

Note: above doesn't work on Windows so please don't use Windows to run this project (you will get banned from contributing if you use Windows /j)

2. cargo watch --ignore 'src/html.rs' -x run

and docker probably doesn't work (it never works)