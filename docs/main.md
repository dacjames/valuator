# Financial Modeler

## Data Model

A **Model** contains one of more **Boards**. Each **Board** contains zero or more **Tiles**.

## Board

A board holds a group of related tiles. You can think of a board like a sheet in a spreadsheet. The difference is that instead of representing one sheet, a Boards contains several mini-sheets.

Boards can be referenced as `#'Financial Model'`).

## Tile

Tiles are polymorphic. A **TableTile** is the default Tile type. Other Tiles include: **GraphTile**, **QueryTile**, **PivotTile**.

Tiles are referenced by name (`&Mortgage, &'Food Prices'`) or by numeric id (`&21`). Absolute tile references are supported: `&$Mortgage` will reference the tile within the current board rather than the new board when copied or moved.

### TableTile

A TableTile contains a table with labeled **Rows** and **Columns**. Row and Column labels can be customized.

Rows are labeled with a lower case letter. Colums are labeles with an upper case letter. Columns/Rows past 26 get repeated letters, like `aa`, `ab`, `ac`.


The table contains **Cells**, which are referenced by the following types of **References**.

#### References

- **Position** (`[0,1]`): 0-based positional index.
  - Supports Python compatible slice syntax: `[1, 3:10]`
  - Column and row indexes may be omitted, defaulting to `0`. 
  `[1] == [1,0]` and `[,1] == [0, 1]`
- **Address** (`{a,A}`, `@{pizza,Price}`): Alphabetical lower and upper case row and column, or custom row/column labels.
  - Positional references can by embedded in addresses and vice versa: `{[:], Price} == @[{Price}, 0:2]`
- **Relative** position (`-[2,3]`, `-[-1,-2]`): negative position relative to the current cell. To refer to a positive relative position, negate the indexes. 
  - `-[0]` and `-[]` both refers to the current cell.
- **Shorthand** address (`@aA, @Aa, @A1, @1A`): Uses the reference format similar to spreadsheet applications. 
  - Prefer to use the alabetical row/columnn labelling over the compatability numbered.
  - This avoids confusion between 1-based row labels and 0-based positional indes.

The `$` symbol can be applied to any reference in the same way as a spreadhseet appliction. `[$0, $2] * {Tomatoes, $Cost} + ($aM * B17)`

### Composing References

References can be composed.

- `#'Financial Model'&Mortage{Price}`
- `&Mortgage{Price} + &Property{Cash} + &Rehab{Duration}`


## Cells

**Cells** are where the magic happens. A cell is composed of a **Value**, a **Formula**, a **Style**, and some additional **Metadata** such as *History*, and *Dependencies*. 


### Data Types

Unlike spreadsheets, cells values contain can contain few different types:

Type|Description|Examples
-|-|-
`Number`|A real number, computed with proper decimal semantics|`1`<br/>`3.14159`
`Boolean`|True or False|`true`<br>`F`
`Float`| A 64bit floating point number. Mainly used as an optimization for Numbers|`Float(2.0)`
`Int`|An integer|`Int(1)`
`String`| Text that should not be interpretted as an expression.|`"-[1] * {Item,Price}"`<br/> `'Hello, World'`<br/> `'a:\' b:"'`
`List`|A collection of values of the same type|`1,2,3`
`Arrays`|N-Dimensional arrays of values. 1-dimensional arrays are similar to lists, but |`1,4,7;2,5,8`<br>`1,2,3; # 1x3 Array `<br>`1;2;3 # 3x1`
`Record`|A collection of named values. Keys and Values are ordered by insertion order|`name:"Daniel",id:17`
`Empty`|All collections share a common "empty" value. The type of the value can usually be inferred. The type constructor (ex: `List()`) can be used to be explicit. |`()`<br>`Array()` 
*Special Types*|Special Types do not have a syntax. They can only be created by calling a constructor function.|`OrderedMap(b:5; a:10)`<br>`SparseArray.fill(0, (100,300))`

### Constants

Constant|Meaning|example
-|-|-
e|Euler's number|`-1 == e^π`
i|Imaginary Numbers|`-1^(1/2) == i^2`
π<br>pi|3.14159...|`2*π == 𝜏`
𝜏<br>tau|The true circle constant|`1 == e^𝜏`
∞<br>infinity|Positive inifinity|`1 < ∞`
-∞<br>-infinity|Negative infinity|`0 > -∞` 
T<br>true| Boolean True|`T` 
F<br>false| Boolean False|`F` 

### Operators

Cell formulas can include operators. valuator provides a wide range of builtin operators and allows custom operators to be defined by the user.

#### Math Operators

Operator|Meaning|Function Equivalent|Example
-|-|-|-
`+`|Addition|math.Add|`3 + 17`
`-`|Subtraction|math.Subtract|`2 - 2`
`-`|Negation|math.Negate|`-1 - -2`
`*`|Multiplication|math.Multiply|`12 * 13`
`*`|Dot Product|math.DotProduct|`1;2 * 2,3`
`**`|Cross Product|math.CrossProduct|`1;2 ** 2;3`
`/`|Division|math.Divide|`1/2`
`//`|Integer Division|math.IntDivide|`1//2`
`^`|Power|math.Power|`2^13`

#### Comparision Operators

Operator|Meaning|Function Equivalent|Example
-|-|-|-
`<`|Less Than|compare.LT|`1<2`
`>`|Greater Than|compare.GT|`3>0`
`<=`|Less Than or Equal|compare.LTE|`1<=1`
`>=`|Greater Than or Equal|compare.GTE|`1>=1`
`==`|Equal To|compare.EQ|`13==13`
`!=`|Not Equal|compare.NEQ|`8!=8`

#### Boolean Operators

Operator|Meaning|Function Equivalent|Example
-|-|-|-
`and`|Boolean And|boolean.And|T and T
`or`|Boolean Or|boolean.Or|true or false
`not`|Boolean Negation|boolean.Not| not true
`xor`|Boolean Exclusive Or|boolean.Xor| true xor F

#### Collection Operators

Operator|Meaning|Function Equivalent|Example
-|-|-|-
`++`|Concatenate|collection.Concat|`1,2,3 ++ 4,5,6`<br/>`1;4 ++ 2;5 ++ 3;6`<br>`'hello ' ++ 'world'`
`**`|Extention|collection.Extend|`('a',) ** 3 == ('a','a','a')`<br>`'a' ** 3 == 'aaa'`<br>`1;2 ** 2 == 1,1;2,2`
`union`|Set union|collection.Union|`1,2,3 union 2,3,3,7`
`inter`|Set intersection|collection.Intersection|`1,2,3 inter 3,4,5`
`in`|Collection contains element|collection.Contains| `1 in 1,2,3`<br>`"f" in "foo"`<br>
`-`|Set difference|collection.Difference|`1,2 - 2`<br>`1,2,3 - 2,3`

## Expressions

Expression|Description|Example
-|-|-
`match`|Pattern matching. Used as the primary control flow expression|`match -[] < 0`<br>`as T colors.black`<br>`as F colors.red`
`if`|Sugar for matching on booleans|`if -finance.ipmt(&Loan{payment}) > &Model{'price target'} colors.green `<br>`else if true colors.red else colors.orange`
*Slicing*|`0,1`
