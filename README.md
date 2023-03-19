Effect handlers in Rust.

Inspired by <https://without.boats/blog/the-registers-of-rust/>, Rust can be considered to have
3 well known effects:
* Asynchrony
* Iteration
* Fallibility

This is currently modelled in Rust using 3 different traits:
* [`Future`](std::future::Future)
* [`Iterator`]
* [`Try`](std::ops::Try)

The "Keyword Generics Initiative" have stirred up a little bit of controversy lately
by proposing some syntax that allows us to compose asynchrony with other traits in a generic
fashion. To put it another way, you can have an [`Iterator`] that is "maybe async" using syntax.

I find the idea interesting, but I think the syntax causes more confusion than it is useful.

I propose the [`Effective`] trait. As I previously mention, there are 3 effects. This
trait models all 3. Instead of _composing_ effects, you can _subtract_ effects.

For instance, [`Future`](std::future::Future) is [`Effective`] - [`Iterator`] - [`Try`](std::ops::Try):

```rust,ignore
impl<E> Future for Wrapper<E>
where
    E: Effective<
        // Fallibility is turned off
        Failure = Infallible,
        // Iteration is turned off
        Produces = Single,
        // Asynchrony is kept on
        Async = Async
    >,
{
    type Output = E::Item;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}
```

## Coloring problem

There's a well known blog post in the JS ecosystem called
["What color is your function?"](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/).
It makes the claim that async functions in JS are a different color to 'non-async' functions.
I don't think the 'color' analogy really works in that case, since colors are known to be mixed.

However, it works perfectly with `Effective`.

```rust,ignore
// Red + Green + Blue
trait TryAsyncIterator = Effective<Failure = Error, Produces = Multiple, Async = Async>;

// Cyan (Blue + Green)
trait AsyncIterator = Effective<Failure = Infallible, Produces = Multiple, Async = Async>;
// Magenta (Red + Blue)
trait TryFuture = Effective<Failure = Error, Produces = Multiple, Async = Async>;
// Yellow (Green + Red)
trait TryIterator = Effective<Failure = Error, Produces = Multiple, Async = Blocking>;

// Red
trait Try = Effective<Failure = Error, Produces = Multiple, Async = Async>;
// Green
trait Iterator = Effective<Failure = Infallible, Produces = Multiple, Async = Blocking>;
// Blue
trait Future = Effective<Failure = Infallible, Produces = Single, Async = Async>;

// Black
trait FnOnce = Effective<Failure = Infallible, Produces = Single, Async = Blocking>;
```

# Examples:

There are a lot of `map`-style functions. [`Iterator::map`],
[`Option::map`], [`FuturesExt::map`](futures_util::future::FutureExt::map),
[`TryStreamExt::map_ok`](futures_util::stream::TryStreamExt::map_ok).

They all do the same thing, map the success value to some other success value.
[`EffectiveExt`] also has a [`map`](EffectiveExt::map) method, but since [`Effective`]
can model all of those effects, it only needs a single method.

## Try:

```rust
use effective::{impls::EffectiveExt, wrappers};

// an effective with just fallible set
let e = wrappers::fallible(Some(42));

let v: Option<i32> = e.map(|x| x + 1).try_get();
assert_eq!(v, Some(43));
```

## Futures:

```rust
# async fn foo() {
use effective::{impls::EffectiveExt, wrappers};

// an effective with just async set
let e = wrappers::future(async { 0 });

let v: i32 = e.map(|x| x + 1).shim().await;
# }
```

## Iterators:

```rust
use effective::{impls::EffectiveExt, wrappers};

// an effective with just iterable set
let e = wrappers::iterator([1, 2, 3, 4]);

let v: Vec<i32> = e.map(|x| x + 1).collect().get();
assert_eq!(v, [2, 3, 4, 5]);
```

## Combined:

```rust
# async fn foo() {
use effective::{impls::EffectiveExt, wrappers};

async fn get_page(x: usize) -> Result<String, Box<dyn std::error::Error>> {
    /* insert http request */
# Ok(x.to_string())
}

// an effective with just iterable set
let e = wrappers::iterator([1, 2, 3, 4])
    .map(get_page)
    // flatten in the async effect
    .flatten_future()
    // flatten in the fallible effect
    .flatten_fallible();

let v: Vec<usize> = e.map(|x| x.len()).collect().unwrap().shim().await;
# }
```

You'll also notice in that last example, we `map` with a fallible async function, and we
can use `flat_map`+`flatten_fallible` to embed the output into the effective directly.

This lets you **add** effects.

We can also **subtract** effects, we can see this in the `collect` method, but there are more.

```rust,ignore
// Effective<Failure = Infallible, Produces = Multiple, Async = Blocking>
let e = wrappers::iterator([1, 2, 3, 4]);

// still the same effects for now...
let e = e.map(get_page);

// Effective<Failure = Infallible, Produces = Multiple, Async = Async>
// We've flattened in in the 'async' effect.
let e = e.flatten_future();

// Effective<Failure = Box<dyn std::error::Error>, Produces = Multiple, Async = Async>
// We've flattened in in the 'fallible' effect.
let e = e.flatten_fallible();

let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();

// Effective<Failure = Box<dyn std::error::Error>, Produces = Multiple, Async = Blocking>
// We've removed the async effect.
let e = e.block_on(runtime);

// Effective<Failure = Box<dyn std::error::Error>, Produces = Single, Async = Blocking>
// We've removed the iterable effect.
let e = e.collect();

// Effective<Failure = Infallible, Produces = Single, Async = Blocking>
// We've removed the fallible effect.
let e = e.unwrap();

// no more effects, just a single value to get
let e: Vec<_> = e.get();
```

# North Star

Obviously this library is quite complex to reason about. I think it is a good pairing to
keyword-generics.

There should be a syntax to implement these concepts but I think the underlying trait is a good
abstraction.

Similar to how:
* `async{}.await` models a [`Future`](std::future::Future),
* `try{}?` models a [`Try`](std::ops::Try),
* `for/yield` models an [`Iterator`]

These syntax elements could be composed to make application level [`Effective`] implementations.

```rust,ignore
async try get_page(after: Option<usize>) -> Result<Option<Page>, Error> { todo!() }

// `async` works as normal, allows the `.await` syntax
// `gen` allows the `yield` syntax, return means `Done`
// `try` allows the `?` syntax.
async gen try fn get_pages() -> Result<Page, Error> {
    let Some(mut page) = get_page(None).await? else {
        // no first page, exit
        return;
    };

    loop {
        let next_page = page.next_page;

        // output the page (auto 'ok-wrapping')
        yield page;

        let Some(p) = get_page(Some(next_page)).await? else {
            // no next page, exit
            return;
        };

        page = p;
    }
}

// This method is still `async` and `try`, but it removes the 'gen` keyword
// because internally we handle all the iterable effects.
async try fn save_pages() -> Result<(), Error> {
    // The for loop is designed to handle iterable effects only.
    // `try` and `await` here tell it to expect and propagate the
    // fallible and async effects.
    for try await page in get_pages() {
        page.save_to_disk()?
    }
}
```

With adaptors, it would look like

```rust,ignore
let get_pages = wrappers::unfold(None, |next_page| {
    wrappers::future(get_page(next_page)).flatten_fallible().map(|page| {
        page.map(|| {
            let next_page = page.next_page;
            (page, Some(next_page))
        })
    })
});

let save_pages = get_pages.for_each(|page| {
    wrappers::future(page.save_to_disk()).flatten_fallible()
});
```
