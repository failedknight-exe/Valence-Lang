# Valence

A programming language I built from scratch in Rust.

## What is Valence?

Valence...

It mainly was made (and is still being made!) to connect languages easily, securly, and kepp the connection fast!
It can do MANY more things and all is detailed below!

## Install
### Pre-Built (Windows)
Take cor.exe and val.exe from 
[Releases](https://github.com/failedknight-exe/Valence-Lang/releases)

Drop it anywhere and then make sure you add to path 

### Your First Project

val init MyProject
cd MyProject
cor run

That's it.
# Valence Command Reference & Applications

## CLI
```bash
cor                    # Launch standard runtime environment
cor run <file.cor>     # Auto-find and run bridge.cor or a specific file
cor morph <file.cor> -o app.exe  # Compile to a native binary
cor check <file.cor>   # Check file syntax without execution
val init <project_name> # Scaffold a new project structure
```

---

## Language Core

### Variables
```valence
varL x = 10         // Local variable - bounded to scope
varG config = true  // Global variable - accessible everywhere
const MAX = 100     // Immutable constant value

| |                 // Explicit null/empty value indicator

print("hi $name")   // String interpolation syntax
```

### Control Flow
```valence
// Conditional Branching
check (cond) { }
orCheck (cond) { }
else { }

// Loops
circle i (n) { }     // Standard numeric loop (0 to n-1)
circle forever { }   // Infinite execution loop
shatter              // Break loop entirely
skip                 // Skip current iteration and move to next

// Pattern Matching
judge (val) {
  "a" => ...,
  _   => ...
}

// Advanced Flow Control
rewind(n)             // Time-travel mechanism: roll back the last n states
protect (a, b) { }    // Safe scoped executions
attempt { } rescue (err) { } always { } // Exception framework
guard (cond) { }      // Pre-condition checking assertion
```

---

## Standard Libraries & Built-ins

### Type Utilities
```valence
type(v)      // Evaluate native data type of value
toInt(v)     // Cast target value to integer
toFloat(v)   // Cast target value to floating point
toString(v)  // Cast target value to string literal
toBool(v)    // Cast target value to boolean
```

### Data Structures

#### Arrays
`[1, 2, 3]`
* `.len()` — Returns array size.
* `.first()` — Access first item.
* `.last()` — Access terminal item.
* `.isEmpty()` — Boolean state check.
* `.has(v)` — Check if item is inside.
* `.indexOf(v)` — Find target index.
* `.join(sep)` — Create a single string.
* `.slice(a,b)` — Cut specific range.
* `.push(v)` — Inject to terminal position.
* `.pop()` — Extract and remove last item.
* `.reverse()` — Flip internal array order.
* `.sort()` — Standard numeric/string sorting.
* `.clear()` — Wipe structure empty.

#### Maps
`{ key: val }`
* `.keys()` — Array of property keys.
* `.values()` — Array of property values.
* `.size()` — Count of element pairs.
* `.has(k)` — Evaluate key existence.
* `.get(k)` — Pull mapped key value.
* `.delete(k)` — Erase key-value block.

### Strings
* `.len()` — Total character length.
* `.upper()` — Convert to uppercase.
* `.lower()` — Convert to lowercase.
* `.trim()` — Strip edge white spaces.
* `.reverse()` — Invert text sequence.
* `.contains(s)` — Check substring.
* `.startsWith(s)` — Matches start string.
* `.endsWith(s)` — Matches end string.
* `.replace(a,b)` — Replace substring variables.
* `.split(sep)` — Fragment string into array.
* `.slice(a,b)` — Substring extract by indices.
* `.charAt(i)` — Isolate targeted index character.
* `.repeat(n)` — Duplicate string n times.
* `.capitalize()` — Uppercase starting character.
* `.camelCase()` — Parse string to camel case.
* `.snakeCase()` — Parse string to snake case.
* `.padLeft(n,c)` — Pad left side with character.
* `.padRight(n,c)` — Pad right side with character.
* `.isNumeric()` — Regex match for digits only.
* `.isAlpha()` — Regex match for letters only.
* `.isEmail()` — Native email syntax parsing.
* `.wordCount()` — Extract text word totals.
* `.truncate(n)` — Limit character length gracefully.
* `.slug()` — Parse text for URL slug formats.
* `.urlEncode()` — Apply standard URL safety masks.
* `.urlDecode()` — Extract dynamic raw string from URL.

---

## Native Libraries

### Math Engine (`math`)
```valence
math.sqrt(n)          math.abs(n)           math.floor(n)         math.ceil(n)          
math.round(n)         math.random(min,max)  math.max(a,b)         math.min(a,b)         
math.pow(b,e)         math.sin(n)           math.cos(n)           math.tan(n)           
math.log(n)           math.log10(n)         math.pi               math.e
```

### Terminal Colors (`paint`)
```valence
paint.red(t)          paint.green(t)        paint.yellow(t)       paint.blue(t)         
paint.magenta(t)      paint.cyan(t)         paint.bold(t)
```

### System Operations (`system`)
```valence
system.os()           system.arch()         system.env("K")       system.exec("cmd")    
system.cwd()          system.exit(code)     system.sleep(ms)      system.args()
```

### File I/O (`file`)
```valence
file.read(path)       file.write(path, data) file.append(path, data)
file.exists(path)     file.delete(path)
```

### Networking (`http`)
```valence
http.get(url)                http.post(url, body)         http.put(url, body)          
http.delete(url)             http.status(url)             http.download(url, dest)     
http.getJson(url)            http.getH(url, headers)      http.postH(url, body, headers)
```

### Storage Engine (`db`)
```valence
db.open(path)         db.set(k,v)           db.get(k)             db.has(k)             
db.delete(k)
```

### Security (`crypto`)
```valence
crypto.hash(data)     crypto.uuid()         crypto.randomInt(min,max)
crypto.randomBytes(n) crypto.base64Encode(s) crypto.base64Decode(s)
```

### Temporal Utilities (`date`)
```valence
date.now()            date.today()          date.time()           date.year()           
date.month()          date.day()            date.hour()           date.minute()         
date.second()         date.timestamp()      date.format(fmt)      date.dayName()        
date.monthName()      date.micro_time()     date.nano_time()
```

### Data Serialization (`json`)
```valence
json.stringify(v)     json.parse(s)
```

### Interoperability & Concurrency (`vbp` / `vault`)
```valence
// Polyglot Engine Bonds
bond "py:script.py" as py
bond "java:App.java" as gui

// Multiprocessing Virtual Blueprint Profile Control
vbp.sync()
vbp.workers
vbp.kill

// Micro-threading
weave (n) { }

// Memory Block Allocations
vault.alloc(bytes)
vault.write(slot, data)
vault.read(slot)
vault.free(slot)
```

---

## Function Architecture

```valence
// Callable Definition
func [callable] name(a, b) {
  reply a + b
}
name(1, 2)

// Daemon & Loop Automation Variants
func [auto] tick() { }      // Automatically schedules lifecycle events
func [forever] loop() { }  // Dedicated automated runtime pipeline loop

// Lifecycle Triggers
trigger name[time](ms)     // Fires continuously on elapsed interval
trigger name[when](cond)   // Fires state condition loop matching criteria
trigger name[once](cond)   // Single execution gatekeeper evaluation
async { }                  // Async task framework blocks

// Workspace Actions
rest(ms)                   // Micro pause execution
wait(ms)                   // Absolute thread hold wait states
use "module.cor"           // Package engine import directive
summon name                // Variable bubble hoisting extraction
input("prompt")            // User terminal capture wrapper
```

---

## Reference Sandbox Applications

### App 1 — Task Manager
```valence
varL tasks = [
  { id: 101, title: "  redesign website  ", priority: "high", done: false },
  { id: 102, title: "build compiler", priority: "critical", done: true },
  { id: 103, title: "record youtube video", priority: "medium", done: false }
]

print("=== VALENCE TASK MANAGER ===")

circle i (tasks.len()) {
  varL item = tasks[i]
  item.title = item.title.trim().capitalize()

  varL status_tag = ""
  judge (item.done) {
    true => status_tag = "[DONE]",
    _    => status_tag = "[PENDING]"
  }

  print(status_tag + " " + item.title + " | Priority: " + item.priority.upper())
}

print("Total Tasks: " + toString(tasks.len()))
```

### App 2 — Bank + Rewind Fraud Rollback
```valence
varL account = {
  owner: "raian ghani",
  balance: 2500,
  tier: "gold"
}

account.owner = account.owner.trim().capitalize()

print("Account: " + account.owner)
print("Start Balance: \$" + toString(account.balance))

account.balance = account.balance - 500
print("After -500: " + toString(account.balance))

account.balance = account.balance - 1200
print("After -1200: " + toString(account.balance))

print("--- FRAUD DETECTED: rewind(2) ---")
rewind(2)

print("Restored Balance: \$" + toString(account.balance))

varL risk = "safe"
judge (risk) {
  "safe" => print("Security: ALL CLEAR"),
  "risk" => print("Security: LOCKED"),
  _      => print("Security: UNKNOWN")
}
```
## Built With

Rust.

## License

GNU General Public License v3.0
