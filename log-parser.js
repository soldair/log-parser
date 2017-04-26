var c = 0
process.stdin.on('data',function(buf){
  for(var i=0;i<buf.length;++i){
    ++c
  }
})
