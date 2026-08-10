# Connect

A programming language I built from scratch in Rust.
No frameworks. No shortcuts. Just pure code.

Born out of boredom. Built out of love.

## What is Connect?

Connect is an interpreted language that runs on a 
Rust-native engine. It has its own lexer, parser, 
and evaluator. Everything from variables to functions 
to file I/O works out of the box.

No pip install. No npm. No drama.
Just write .cor files and run them.

## Install

### Build From Source

git clone https://github.com/failedknight-exe/Connect-Lang
cd Connect-Lang
cargo build --release

### Pre-Built (Windows)
Grab cor.exe and crh.exe from 
[Releases](https://github.com/failedknight-exe/Connect-Lang/releases)

Drop them somewhere. Add to PATH. Done.

### Your First Project

1. crh init MyProject
2. cd MyProject
3. cor run

That's it. You're coding in Connect.
## The Basics

### Variables

Connect has three types of variables.
Each one has its place. Each one has its rules.

varL name = 'Knight'          // Local - stays in its scope
varG score = 0                // Global - lives everywhere
const PI = 3.14159            // Constant - never changes

update score = 100            // Change what you declared
varL a, b, c = 10, 20, 30    // Multiple at once
varL empty = | |              // Null (nothing between walls)
varL debt = -42               // Negatives work

### Summon

Connect's secret weapon.
varL is locked to its scope. Functions can't see it.
Unless you summon it.

varL secret = 'hidden treasure'

// This fails. secret is not here.
func[callable] noLuck() {
    print(secret)              // ERROR: not found
}

// summon pulls it in.
func[callable] gotIt() {
    summon secret
    print(secret)              // Works: hidden treasure
}

// varG doesn't need summon. Already everywhere.
varG visible = 'always here'
func[callable] easyAccess() {
    print(visible)             // Works. No summon needed.
}

## Data Types

42                    // Integer
3.14                  // Float
'hello'               // String
`he said 'wow'`       // Template string
true                  // Boolean
false                 // Boolean
[1, 2, 3]            // Array
{name: 'Knight'}     // Map
| |                   // Null

## Strings

Single quotes. That's the Connect way.
Need quotes inside? Use backticks.

varL text = 'Failed Knight'

text.len                        // 13
text.upper                      // FAILED KNIGHT
text.lower                      // failed knight
text.trim                       // removes whitespace
text.reverse                    // thginK deliaF
text.repeat(2)                  // Failed KnightFailed Knight
text.contains('Knight')         // true
text.startsWith('Failed')       // true
text.endsWith('Knight')         // true
text.slice(0, 6)                // Failed
text.charAt(0)                  // F
text.replace('Failed', 'Epic')  // Epic Knight
text.split(' ')                 // [Failed, Knight]

// Escape sequences
print('Line 1\nLine 2')
print('Tab\there')

// Template strings
varL msg = `He said 'Connect is fire'`

## Arrays

Square brackets. Simple.

varL nums = [5, 3, 1, 4, 2]

nums[0]                 // 5
nums.len                // 5
nums.first              // 5
nums.last               // 2
nums.isEmpty            // false
nums.has(3)             // true
nums.indexOf(4)         // 3
nums.push(6)            // adds to end
nums.pop()              // removes last
nums.sort               // [1, 2, 3, 4, 5]
nums.reverse            // [5, 4, 3, 2, 1]
nums.join(' - ')        // 5 - 4 - 3 - 2 - 1
nums.slice(1, 3)        // [4, 3]
nums.clear              // []

## Maps

Key-value pairs. Like a dictionary but cooler.

varL user = {name: 'Knight', age: 14, lang: 'Connect'}

user.name               // Knight
user.age                // 14
user.size               // 3
user.keys               // [name, age, lang]
user.values             // [Knight, 14, Connect]
user.has('name')        // true
user.has('email')       // false
user.get('lang')        // Connect
user.delete('age')      // removes age

## Math

Connect follows PEMDAS automatically.
No thinking required.

10 + 5                  // 15
10 - 3                  // 7
4 * 3                   // 12
20 / 4                  // 5
10 % 3                  // 1
2 ** 8                  // 256

// PEMDAS just works
2 + 3 * 4               // 14 (not 20)
(2 + 3) * 4             // 20

// Chain as much as you want
1 + 2 + 3 + 4 + 5       // 15

### Math Library

math.sqrt(144)           // 12
math.abs(-99)            // 99
math.floor(3.9)          // 3
math.ceil(3.1)           // 4
math.round(3.5)          // 4
math.random(1, 100)      // random number
math.max(50, 100)        // 100
math.min(50, 100)        // 50
math.pow(2, 8)           // 256
math.pi                  // 3.14159...
math.e                   // 2.71828...
math.sin(x)              // also cos, tan
math.log(x)              // also log10

## Conditions

No if/else here. Connect uses check.

check (x > 5) {
    print('Big')
}

check (x == 10) {
    print('Ten')
} else {
    print('Not ten')
}

// Chain with orCheck
check (score >= 90) {
    print('A')
} orCheck (score >= 80) {
    print('B')
} orCheck (score >= 70) {
    print('C')
} else {
    print('F')
}

### Logical Operators

check (x > 5 and x < 10) {
    print('Between 5 and 10')
}

check (x == 0 or x == 1) {
    print('Binary')
}

check (not ready) {
    print('Not ready yet')
}

### Comparisons

a == b      // equal
a != b      // not equal
a > b       // greater
a < b       // less
a >= b      // greater or equal
a <= b      // less or equal

## Loops

Connect uses circle. Because loops go around.
The loop name becomes your counter. Clean.

circle i(5) {
    print(i)
}
// Output: 0 1 2 3 4

// Name it whatever
circle count(10) {
    print(count)
}

// Break out with shatter
circle i(100) {
    check (i == 5) {
        shatter
    }
    print(i)
}
// Output: 0 1 2 3 4

// Skip with skip
circle i(5) {
    check (i == 2) {
        skip
    }
    print(i)
}
// Output: 0 1 3 4

// Nested loops
circle x(3) {
    circle y(3) {
        print(toString(x) + ',' + toString(y))
    }
}

## Functions

Functions in Connect have types.
Each type has its own behavior.

// Callable - your standard function
func[callable] add(a, b) {
    reply a + b
}
print(add(5, 10))               // 15

// No params? No problem.
func[callable] greet() {
    print('Hello!')
}
greet()

// Recursion works
func[callable] factorial(n) {
    check (n <= 1) {
        reply 1
    }
    reply n * factorial(n - 1)
}
print(factorial(10))             // 3628800

// Onetime - runs once. Then it retires.
func[onetime] init() {
    print('Setup complete')
}
init()                           // Works
init()                           // ERROR: already ran

// Functions calling functions
func[callable] square(n) {
    reply n * n
}
func[callable] sumSquares(a, b) {
    reply square(a) + square(b)
}
print(sumSquares(3, 4))          // 25

## Type System

type(42)                 // integer
type(3.14)               // float
type('hi')               // string
type(true)               // boolean
type([1, 2])             // array
type({a: 1})             // map
type(| |)                // null

toInt('42')              // 42
toFloat('3.14')          // 3.14
toString(42)             // '42'
toBool(1)                // true

## File I/O

Read. Write. Delete. Simple.

file.write('data.txt', 'Hello World')
varL content = file.read('data.txt')
file.append('log.txt', 'New entry')
file.exists('data.txt')          // true
file.delete('data.txt')

## Modules

Import other .cor files.
Or generate code and import it.

use 'utils.cor'

file.write('gen.cor', `func[callable] hi() { reply 'Generated!' }`)
use 'gen.cor'
print(hi())                      // Generated!

## Input / Output

print('Hello World')
print(42)
print([1, 2, 3])

varL name = input('Name: ')
print('Hello ' + name)

## Timing

rest(1000)               // Pause 1 second
wait(500)                // Pause 0.5 seconds

## Comments

// Single line

/* Multi-line
   comment block */

## CLI Commands

cor run                  // Auto-find and run bridge.cor
cor run file.cor         // Run specific file
cor check file.cor       // Check syntax
cor help                 // Show commands
cor version              // Show version

crh init ProjectName     // Scaffold a new project
crh help                 // Show commands

## Project Structure

MyProject/
    src/
        bridge.cor       // Your code starts here
    modules/             // Packages go here
    data.toml            // Project config

## Example: Number Guessing Game

varG secret = math.random(1, 100)
varG won = false

circle attempt(10) {
    check (won) { shatter }
    varL guess = toInt(input('Guess 1-100: '))
    check (guess == secret) {
        print('You got it!')
        update won = true
    } orCheck (guess < secret) {
        print('Higher!')
    } else {
        print('Lower!')
    }
}

check (not won) {
    print('Game over! It was ' + toString(secret))
}

## Example: Grade Calculator

varL scores = [85, 92, 78, 95, 88]
varL total = 0

circle i(scores.len) {
    update total = total + scores[i]
}

varL avg = total / scores.len

check (avg >= 90) { print('A') }
orCheck (avg >= 80) { print('B') }
orCheck (avg >= 70) { print('C') }
else { print('F') }

## Built With

Rust. From scratch.
No frameworks. No hand-holding.
Just mass research, mass errors, and mass dedication.

## License

GNU General Public License v3.0
