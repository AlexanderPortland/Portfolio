# Portfolio Testing

## General Workflow
*build using `make frontend` and then `make backend` in a seperate terminal*
1. log in as admin
    - visit `http://[::1]:5173/admin/login`
    - log in using id: `1`, password: `hello`
2. make a new applicant
    - THE RULES
        - all candidate ids must start with valid subject prefix (101, 102, 103)
            - and have >=6 numbers
            - I think there's some other checksum too but idk what it is
        - all candidate government id's ('Rodné číslo's) must be valid theorhetical [czech ids](https://cs.wikipedia.org/wiki/Rodn%C3%A9_%C4%8D%C3%ADslo#Kontroln%C3%AD_%C4%8D%C3%ADslice) (10 digits) i just use `736028/5163` from the wikipedia page
            - sum of the digits in the odd places - sum of the digits in the even places must be divisible by 11???
                - eg `736028/5163`
                - or `739098/6163`
                - or `846028/5163`
3. try to log in as that applicant
    - use id & password given (eg `10222324` & `UB272EQWUIF2`)