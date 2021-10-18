# mcfrs

WOOOOOOOO, another Minecraft commands precompiler!!1

## Why should I care?

Syntax of this precompiler does not suck, as it sticks to couple of simple principles:

- Ease of use, by not defining a ton of new keywords

`.mcf` files (their content is compiled into regular Minecraft's commands) almost
identical with regular `.mcfunction` files, the main diffrence is only that `.mcf`s can
have `scope`s

- Compatibility with vanilla functions' syntax

Because of that writing `.mcf`s is as easy as writing normal functions, its eaven easier!

- Fixing problems, rather than creating new ones

The main problem that I have witnessed while writing vanilla functions was controll flow.
`execute if` and others are relly good commands, however utilizing hevely `@s` with them requires
writing a lot of one use files (functions with litelary 2~3 commands). Because of that
`.mcf` allows for scopes (defined by intendation) witch are awesome.

## Awesome syntax

- If something, run multiple commands

```
execute if entity @a[tag=player] run function
	say There is at least 1 player left!
	give @a[tag=!player] minecraft:diamond_sword
```

- Repeat this for...

```
call check_if_block_nerby minecraft:stone
call check_if_block_nerby minecraft:glass
call check_if_block_nerby minecraft:dirt
call check_if_block_nerby minecraft:stone

generate function check_if_block_nerby $block
	execute if block ~ ~ ~-1 $block if block ~ ~ ~1 $block if block ~-1 ~ ~ $block if block ~1 ~ ~ $block run function
		say $block is nerby!
```

- This single command is really long

```
execute 
back as @a
back at @s
back if score @s score matches 5..
back run kill @s
```
