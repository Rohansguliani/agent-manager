import { SimpleChat } from './components/SimpleChat';
import { ErrorBoundary } from './ErrorBoundary';
import { ToastProvider } from './components/ToastProvider';

export default function SimpleChatPage() {
  return (
    <ErrorBoundary>
      <ToastProvider />
      <SimpleChat />
    </ErrorBoundary>
  );
}

