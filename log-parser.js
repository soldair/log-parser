const fs = require('fs')

const buff = fs.readFileSync('test.log')
console.log(buff.length)
