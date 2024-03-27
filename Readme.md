Inspired by popular frameworks such as Boost serialization it aims to make serialization of
any datatypes as easy as possible. 

# No Configuration Needed
Out of the box it is capable of (de-)serialization of any standard datatype

# Why another framework instead of using Serde or other already existing ones?
Serde particular is a great, stable framework that works very well and in fact we encourage you to use it when in 
doubt. However we designed MUU as Serde couldn't fulfill two of our requirements very well: 
* Backward compatibility with already established data and file formats
* Versioning

If you have an existing SDK with some quirks that you cannot undo (like mixing little and big endian) all the framework will fail you.