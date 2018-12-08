var N = null;var searchIndex = {};
searchIndex["divbuf"]={"doc":"Recursively divisible buffer class","items":[[3,"Chunks","divbuf","The return type of `DivBuf::into_chunks`",N,N],[3,"ChunksMut","","The return type of `DivBufMut::into_chunks`",N,N],[3,"DivBufShared","","The \"entry point\" to the `divbuf` crate.",N,N],[3,"DivBuf","","Provides read-only access to a buffer.",N,N],[3,"DivBufMut","","Provides read-write access to a buffer",N,N],[11,"capacity","","Returns the number of bytes the buffer can hold without reallocating.",0,[[["self"]],["usize"]]],[11,"is_empty","","Returns true if the `DivBufShared` has length 0",0,[[["self"]],["bool"]]],[11,"len","","Returns the number of bytes contained in this buffer.",0,[[["self"]],["usize"]]],[11,"try_const","","Try to create a read-only [`DivBuf`] that refers to the entirety of this buffer.  Will fail if there are any [`DivBufMut`] objects referring to this buffer.",0,[[["self"]],["result",["divbuf","str"]]]],[11,"try_mut","","Try to create a mutable `DivBufMut` that refers to the entirety of this buffer.  Will fail if there are any [`DivBufMut`] or [`DivBuf`] objects referring to this buffer.",0,[[["self"]],["result",["divbufmut","str"]]]],[11,"with_capacity","","Creates a new, empty, `DivBufShared` with a specified capacity.",0,[[["usize"]],["self"]]],[11,"into_chunks","","Break the buffer up into equal sized chunks",1,[[["self"],["usize"]],["chunks"]]],[11,"is_empty","","Returns true if the `DivBuf` has length 0",1,[[["self"]],["bool"]]],[11,"len","","Get the length of this `DivBuf`, not the underlying storage",1,[[["self"]],["usize"]]],[11,"slice","","Create a new DivBuf that spans a subset of this one.",1,[[["self"],["usize"],["usize"]],["divbuf"]]],[11,"slice_from","","Creates a new DivBuf that spans a subset of this one, including the end",1,[[["self"],["usize"]],["divbuf"]]],[11,"slice_to","","Creates a new DivBuf that spans a subset of self, including the beginning",1,[[["self"],["usize"]],["divbuf"]]],[11,"split_off","","Splits the DivBuf into two at the given index.",1,[[["self"],["usize"]],["divbuf"]]],[11,"split_to","","Splits the DivBuf into two at the given index.",1,[[["self"],["usize"]],["divbuf"]]],[11,"try_mut","","Attempt to upgrade Self to a writable DivBufMut",1,[[["self"]],["result",["divbufmut","divbuf"]]]],[11,"unsplit","","Combine splitted DivBuf objects back into a contiguous single",1,[[["self"],["divbuf"]],["result",["divbuf"]]]],[11,"freeze","","Downgrade this `DivBufMut` into a read-only `DivBuf`",2,[[["self"]],["divbuf"]]],[11,"into_chunks","","Break the buffer up into equal sized chunks",2,[[["self"],["usize"]],["chunksmut"]]],[11,"is_empty","","Returns true if the `DivBufMut` has length 0",2,[[["self"]],["bool"]]],[11,"len","","Get the length of this `DivBufMut`, not the underlying storage",2,[[["self"]],["usize"]]],[11,"reserve","","Reserves capacity for at least `additional` more bytes to be inserted into the buffer.",2,[[["self"],["usize"]]]],[11,"split_off","","Splits the DivBufMut into two at the given index.",2,[[["self"],["usize"]],["divbufmut"]]],[11,"split_to","","Splits the DivBufMut into two at the given index.",2,[[["self"],["usize"]],["divbufmut"]]],[11,"try_extend","","Attempt to extend this `DivBufMut` with bytes from the provided iterator.",2,[[["self"],["t"]],["result",["str"]]]],[11,"try_resize","","Attempt to resize this `DivBufMut` in-place.",2,[[["self"],["usize"],["u8"]],["result",["str"]]]],[11,"try_truncate","","Shortens the buffer, keeping the first `len` bytes and dropping the rest.",2,[[["self"],["usize"]],["result",["str"]]]],[11,"unsplit","","Combine splitted DivBufMut objects back into a contiguous single",2,[[["self"],["divbufmut"]],["result",["divbufmut"]]]],[11,"into_iter","","",3,[[["self"]],["i"]]],[11,"into","","",3,[[["self"]],["u"]]],[11,"from","","",3,[[["t"]],["t"]]],[11,"try_from","","",3,[[["u"]],["result"]]],[11,"borrow","","",3,[[["self"]],["t"]]],[11,"borrow_mut","","",3,[[["self"]],["t"]]],[11,"try_into","","",3,[[["self"]],["result"]]],[11,"get_type_id","","",3,[[["self"]],["typeid"]]],[11,"into_iter","","",4,[[["self"]],["i"]]],[11,"into","","",4,[[["self"]],["u"]]],[11,"from","","",4,[[["t"]],["t"]]],[11,"try_from","","",4,[[["u"]],["result"]]],[11,"borrow","","",4,[[["self"]],["t"]]],[11,"borrow_mut","","",4,[[["self"]],["t"]]],[11,"try_into","","",4,[[["self"]],["result"]]],[11,"get_type_id","","",4,[[["self"]],["typeid"]]],[11,"into","","",0,[[["self"]],["u"]]],[11,"from","","",0,[[["t"]],["t"]]],[11,"try_from","","",0,[[["u"]],["result"]]],[11,"borrow","","",0,[[["self"]],["t"]]],[11,"borrow_mut","","",0,[[["self"]],["t"]]],[11,"try_into","","",0,[[["self"]],["result"]]],[11,"get_type_id","","",0,[[["self"]],["typeid"]]],[11,"into","","",1,[[["self"]],["u"]]],[11,"to_owned","","",1,[[["self"]],["t"]]],[11,"clone_into","","",1,N],[11,"from","","",1,[[["t"]],["t"]]],[11,"try_from","","",1,[[["u"]],["result"]]],[11,"borrow","","",1,[[["self"]],["t"]]],[11,"borrow_mut","","",1,[[["self"]],["t"]]],[11,"try_into","","",1,[[["self"]],["result"]]],[11,"get_type_id","","",1,[[["self"]],["typeid"]]],[11,"into","","",2,[[["self"]],["u"]]],[11,"from","","",2,[[["t"]],["t"]]],[11,"try_from","","",2,[[["u"]],["result"]]],[11,"borrow","","",2,[[["self"]],["t"]]],[11,"borrow_mut","","",2,[[["self"]],["t"]]],[11,"try_into","","",2,[[["self"]],["result"]]],[11,"get_type_id","","",2,[[["self"]],["typeid"]]],[11,"cmp","","",1,[[["self"],["divbuf"]],["ordering"]]],[11,"cmp","","",2,[[["self"],["divbufmut"]],["ordering"]]],[11,"drop","","",0,[[["self"]]]],[11,"drop","","",1,[[["self"]]]],[11,"drop","","",2,[[["self"]]]],[11,"extend","","",2,[[["self"],["t"]]]],[11,"partial_cmp","","",1,[[["self"],["divbuf"]],["option",["ordering"]]]],[11,"partial_cmp","","",2,[[["self"],["divbufmut"]],["option",["ordering"]]]],[11,"next","","",3,[[["self"]],["option",["divbuf"]]]],[11,"size_hint","","",3,N],[11,"next","","",4,[[["self"]],["option",["divbufmut"]]]],[11,"size_hint","","",4,N],[11,"eq","","",1,[[["self"],["divbuf"]],["bool"]]],[11,"eq","","",1,N],[11,"eq","","",2,[[["self"],["divbufmut"]],["bool"]]],[11,"eq","","",2,N],[11,"as_ref","","",1,N],[11,"as_ref","","",2,N],[11,"from","","",0,N],[11,"from","","",0,[[["vec",["u8"]]],["divbufshared"]]],[11,"from","","",1,[[["divbufmut"]],["divbuf"]]],[11,"clone","","",1,[[["self"]],["divbuf"]]],[11,"hash","","",1,[[["self"],["h"]]]],[11,"hash","","",2,[[["self"],["h"]]]],[11,"deref_mut","","",2,N],[11,"deref","","",1,N],[11,"deref","","",2,N],[11,"fmt","","",3,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",4,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",1,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",2,[[["self"],["formatter"]],["result"]]],[11,"fmt","","",0,[[["self"],["formatter"]],["result",["error"]]]],[11,"borrow","","",1,N],[11,"borrow","","",2,N],[11,"write","","",2,N],[11,"write_all","","",2,N],[11,"flush","","",2,[[["self"]],["result"]]],[11,"borrow_mut","","",2,N]],"paths":[[3,"DivBufShared"],[3,"DivBuf"],[3,"DivBufMut"],[3,"Chunks"],[3,"ChunksMut"]]};
initSearch(searchIndex);
