import React from 'react'
import ReactDOM from 'react-dom/client'
// Use SimpleChatPage for the simplified chat flow
import SimpleChatPage from './SimpleChatPage'
// Original App is still available - uncomment to use it instead
// import App from './App'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <SimpleChatPage />
  </React.StrictMode>,
)

