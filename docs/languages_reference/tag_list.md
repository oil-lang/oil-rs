# Markup tags

Each tag has a specific semantic and should be use only for its purpose. If you need
a generic tag, you probably want a [group](tag_list.md#group) tag.

All tags accept a `class` attribute that stores the class list
as follow:
```xml
<anytag class="classname0 classname1 classname2"></anytag>
```

#### view

**Example:**

```xml
<view name="foo">
      <!-- any tag from below -->
</view>
```

**Context:**

Need to be declared at the root of the markup file. Otherwise, it will be ignored
and the parser will emit a warning.

**Attributes:**

 - `name` contains the name of the view. If the value provided is `main` then this
   view will be the first one to be loaded. The value can't be a data binding.


#### template declaration

**Example:**

```xml
<template name="bar">
      <!-- any tag from below -->
</template>
```

**Context:**

As for the tag `view`, this one need to be at the root level. Otherwise, it will
be interpreted differently.

**Attributes:**

 - `name` contains the name of the template. It can't be a data binding.


#### text-input

**Example:**

```xml
<text-input value="{{tchat.msg}}" key="{{options.keyboard.submitmsg}}"/>
```

**Context:** None

**Attributes:**

 - `value` represent the editable content of the input. This **has** to be a
    data binding. If not, then the tag will be ignored and a warning will be emited.
 - `key` represent the KeyCode that will trigger the user event `UserEvent::Submit`.
    By default, this is the enter key. A data binding is valid here.

#### progress-bar

**Example:**

```xml
<progress-bar value="{{player.xp_pct}}"/>
```

**Context:** None

**Attributes:**

 - `value` contains the current percentage for the progress bar.
   It can be a floating point value between `0.0` and `100.0`.

#### group

**Example:**

```xml
<group></group>
```

**Context:** None

**Attributes:** None

#### button

**Example:**

```xml
<button gotoview="foo" action="bar" key="A"/>
```

**Context:** None

**Attributes:**

 - `gotoview` contains the name of a view that will be pushed in the `Router`'s
   stack after pressing key `key`.
 - `action` contains the name of an [action](../action.md)
 - `key` contains the KeyCode triggering the user event `UserEvent::Submit`.
   By default this is the enter key. As for the `text-input`, this can be a data binding.

#### template inclusion

**Example:**

```xml
<template path="foobar" />
```

**Context:** None

**Attributes:**

 - `path` contains the name of a template that will be included here in the tree.
   You can use a data binding here to emulate tabs.

#### repeat

**Example:**

```xml
<!-- template declaration -->
<template name="friend">
    {{name}} is {{status}}
</template>

<view name="friends">
    <!-- Repeat usage -->
    <repeat iter="{{player.friends}}" template-name="friend"/>
</view>
```

**Context:** None

**Attributes:**

 - `iter` **must** be a data binding. The data binding will be traversed during the
   render phase. For more details see `ContextManager`.
 - `template-name` is the name of the template that will be used to render each element.
   All data bindings defined inside the template will first lookup the value inside
   the current value being rendered and then lookup in the direct more global context.
   That means that in the given example, it as if `{{inventory.items[i].name}}` was
   first being looked up and if it does not exists, then `{{name}}` would be.
