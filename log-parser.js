
var split = require('split2')
var c= 0
var total = 0
var counts = {}

var size = require('prettysize')

// logs are slightly sorted by time. 
// we may get events from a window of about 3 minutes in any given time.
// lets buffer 5 minutes at a time to make sure we count.
var minutes = {}
var buffered = []

var bytes = 0
process.stdin.pipe(split()).on('data',function(l){
    

  bytes += l.length+1
  // bytes
  // service
  var parts = l.split(' ')

  if(parts.length < 7) return;

  var date = parts[0].substr(parts[0].indexOf('>')+1)

  var time = Date.parse(date)
  var minute = time-(time% (1000*60) )
  
  if(!minutes[minute]){
    minutes[minute] = {time:new Date(minute).toJSON(),count:0,size:0}
    buffered.push(minute)
    buffered.sort()
    if(buffered.length > 5){
      var removeMinute = buffered.shift()
      var removedMinute = minutes[removeMinute]
      delete minutes[removeMinute]
      process.stdout.write(JSON.stringify(removedMinute)+"\n")
    }
  }
  
  var counts = minutes[minute]

  var pop = parts[1].substr(6,3)

  var status = parts[7]

  var path = pathToService(parts[6]||'')

  var sizes = parts.slice(parts.length-4)

  var time = unsigned(sizes[0])
  var bodyBytes = unsigned(sizes[1])
  var egress = unsigned(sizes[2])
  var ingress = unsigned(sizes[3])

  
  var pathObj = {}

  total += egress
  if(!counts[path]) counts[path] = {count:0,size:0,status:{},pop:{}};
  pathObj = counts[path]

  if(!pathObj.status[status]) pathObj.status[status] = 0
  if(!pathObj.pop[pop]) pathObj.pop[pop] = {}
  if(!pathObj.pop[pop][status]) pathObj.pop[pop][status] = 0

  pathObj.status[status]++
  pathObj.pop[pop][status]++
  pathObj.count++
  pathObj.size += egress

  counts.count++
  counts.size += egress
  

  ++c 
}).on('end',report)

process.on('SIGINT',report)

function report(){

console.error('count ',bytes)
        
        buffered.forEach(function(minute){
		process.stdout.write(JSON.stringify(minutes[minute])+"\n")
	})
        
	process.exit()
        /*	
	Object.keys(counts).forEach(function(n){
		//console.log(counts[n])
		console.log(n+"\n",Math.floor((counts[n].s/total)*100)+'%\n',size(counts[n].s)+'\n')
	})
        */
	
}


function unsigned(v){
  if((v+'').indexOf('"') > -1){
    v = v.replace(/"/g,'')
  }

  if(v === '-') v = 0
  if(isNaN(+v)) return 0
  v = +v
  if(v < 0) v = 0
  return v
}

function pathToService(path){
	path = path+''
	if(path.indexOf('@') > -1) {
		if(path.indexOf('.tgz') > -1){
			return 'scoped tarball'
		}
		return 'scoped json'
	}
	if(path.indexOf('doc.json') > -1){
		return 'static json'
	}
	if(path.indexOf('doc.min.json') > -1){
		return 'corgi json'
	}
	if(path.indexOf('.tgz') === path.length - 5){
		return 'tarball'
	}
	if(path.indexOf('/-/') === 0) {
		return 'tie-fighter'
	}
	if(path.match(/^\/[^\/]+$/)) {
		return 'json'
        }
	return 'misc'
}

