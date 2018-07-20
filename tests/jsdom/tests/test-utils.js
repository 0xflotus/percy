const jsdom = require('jsdom')
const { JSDOM } = jsdom

module.exports = {
  initDOM: initDOM
}

function initDOM () {
  const dom = new JSDOM('<!DOCTYPE html><body></body></html>')

  global.HTMLDocument = dom.window.HTMLDocument
  global.Element = dom.window.Element
  global.HTMLCollection = dom.window.HTMLCollection
  global.NodeList = dom.window.NodeList

  global.window = dom.window
  global.document = dom.window.document
  global.document = dom.window.document
}
