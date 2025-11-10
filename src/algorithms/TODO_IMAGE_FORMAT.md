have a genereal dynamic image struct (like the one in img, maybe, or even we could just include the img library lol)
along with its native decoder, have `*_from_dynamic` and `*_to_dynamic` for each format, such as png, qoi, jpeg ...
dynamic format stores raw pixels uncompressed

this'll give us enough building blocks to do cool stuff like this:

```sh
$ stackpack e img.png comp.qoi.bsc --using "png_decode -> png_to_dynamic -> qoi_from_dynamic -> bsc"
```

i tested, this can beat png.