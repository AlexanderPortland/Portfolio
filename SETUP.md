*testing was last working properly using alohomora commit: `da25b4eeae38bbca6b9bf443cc8c74b30cd8681b`*
*running was last working properly using alohomora commit: `da25b4eeae38bbca6b9bf443cc8c74b30cd8681b`*
# setup steps
1. setup backend (start from project root)
    - run `export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/` to set db url (or replace with yours)
    - change the `init` variable on line 35 of [`pool.rs`](/api/src/pool.rs) to true (just for the first time running to initialize db)
    - `cargo run`
- if you ever get dependency issues just run `rustup default nightly-2022-09-24`
2. setup frontend
    - in a seperate terminal window go to `frontend` subdirectory
    - run `export PORTFOLIO_API_HOST=127.0.0.1:8000` (or equivalent based on where backend api is hosted)
    - run `rustup default nightly-2022-09-24`
    - `npm install` and then `npm run dev`
3. login as admin
    - navigate to `http://[::1]:5173/admin/login` or `http://localhost:5173/admin/login` or whatever
        - id should be 1 and password should be "hello"
    - now you can create candidates
    - some weird restrictions tho
        - all candidate ids must start with valid subject prefix (101, 102, 103)
        - all candidate government id's ('Rodné číslo's) must be valid theorhetical [czech ids](https://cs.wikipedia.org/wiki/Rodn%C3%A9_%C4%8D%C3%ADslo#Kontroln%C3%AD_%C4%8D%C3%ADslice) (10 digits w sum divisible by 11) i just use `736028/5163` from the wikipedia page