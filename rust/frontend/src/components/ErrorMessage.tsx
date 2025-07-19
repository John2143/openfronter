import React from 'react';

interface ErrorMessageProps {
  message: string;
}

const ErrorMessage: React.FC<ErrorMessageProps> = ({ message }) => (
  <div style={{
    background: '#ffe6e6',
    border: '1px solid #ff4444',
    borderRadius: '8px',
    padding: '16px',
    margin: '20px 0',
    color: '#cc0000',
    textAlign: 'center'
  }}>
    <strong>Error:</strong> {message}
  </div>
);

export default ErrorMessage;
