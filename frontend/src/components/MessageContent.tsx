import React, { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { CopyIcon, CheckIcon } from './icons';
import { theme } from '../styles/theme';

interface MessageContentProps {
  content: string;
  role: 'user' | 'assistant';
}

export const MessageContent: React.FC<MessageContentProps> = ({ content, role }) => {
  const [copiedCodeBlock, setCopiedCodeBlock] = useState<string | null>(null);

  const copyToClipboard = async (text: string, blockId: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedCodeBlock(blockId);
      setTimeout(() => setCopiedCodeBlock(null), 2000);
    } catch {
      // Silently fail
    }
  };

  if (role === 'user') {
    // User messages: simple text rendering (no markdown)
    return (
      <div style={userMessageStyles.content}>
        {content.split('\n').map((line, idx) => (
          <React.Fragment key={idx}>
            {line}
            {idx < content.split('\n').length - 1 && <br />}
          </React.Fragment>
        ))}
      </div>
    );
  }

  // Assistant messages: full markdown rendering
  return (
    <div style={assistantMessageStyles.content}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          code(props: any) {
            const { inline, className, children } = props;
            const match = /language-(\w+)/.exec(className || '');
            const codeString = String(children).replace(/\n$/, '');
            const blockId = `code-${Math.random().toString(36).substr(2, 9)}`;

            return !inline && match ? (
              <div style={codeBlockStyles.container}>
                <div style={codeBlockStyles.header}>
                  <span style={codeBlockStyles.language}>{match[1]}</span>
                  <button
                    onClick={() => copyToClipboard(codeString, blockId)}
                    style={{
                      ...codeBlockStyles.copyButton,
                      ...(copiedCodeBlock === blockId ? codeBlockStyles.copyButtonCopied : {}),
                    }}
                    title="Copy code"
                    aria-label="Copy code block"
                  >
                    {copiedCodeBlock === blockId ? (
                      <>
                        <CheckIcon size={14} style={codeBlockStyles.checkIcon} />
                        <span style={codeBlockStyles.copyText}>Copied</span>
                      </>
                    ) : (
                      <>
                        <CopyIcon size={14} />
                        <span style={codeBlockStyles.copyText}>Copy</span>
                      </>
                    )}
                  </button>
                </div>
                <SyntaxHighlighter
                  style={vscDarkPlus}
                  language={match[1]}
                  PreTag="div"
                  customStyle={codeBlockStyles.syntaxHighlighter}
                >
                  {codeString}
                </SyntaxHighlighter>
              </div>
            ) : (
              <code className={className} style={inlineCodeStyles} {...props}>
                {children}
              </code>
            );
          },
          p: ({ children }) => <p style={paragraphStyles}>{children}</p>,
          h1: ({ children }) => <h1 style={headingStyles.h1}>{children}</h1>,
          h2: ({ children }) => <h2 style={headingStyles.h2}>{children}</h2>,
          h3: ({ children }) => <h3 style={headingStyles.h3}>{children}</h3>,
          ul: ({ children }) => <ul style={listStyles.ul}>{children}</ul>,
          ol: ({ children }) => <ol style={listStyles.ol}>{children}</ol>,
          li: ({ children }) => <li style={listStyles.li}>{children}</li>,
          blockquote: ({ children }) => (
            <blockquote style={blockquoteStyles}>{children}</blockquote>
          ),
          a: ({ href, children }) => (
            <a href={href} target="_blank" rel="noopener noreferrer" style={linkStyles}>
              {children}
            </a>
          ),
          table: ({ children }) => (
            <div style={tableWrapperStyles}>
              <table style={tableStyles}>{children}</table>
            </div>
          ),
          th: ({ children }) => <th style={tableHeaderStyles}>{children}</th>,
          td: ({ children }) => <td style={tableCellStyles}>{children}</td>,
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};

const userMessageStyles = {
  content: {
    whiteSpace: 'pre-wrap' as const,
    wordBreak: 'break-word' as const,
  },
};

const assistantMessageStyles = {
  content: {
    lineHeight: theme.typography.lineHeight.relaxed,
  },
};

const codeBlockStyles = {
  container: {
    margin: `${theme.spacing.md} 0`,
    borderRadius: theme.borderRadius.md,
    overflow: 'hidden' as const,
    backgroundColor: theme.colors.background.primary,
    border: `1px solid ${theme.colors.border.primary}`,
    boxShadow: theme.shadows.md,
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: `${theme.spacing.sm} ${theme.spacing.md}`,
    backgroundColor: theme.colors.background.elevated,
    borderBottom: `1px solid ${theme.colors.border.primary}`,
  },
  language: {
    fontSize: theme.typography.fontSize.xs,
    color: theme.colors.text.tertiary,
    textTransform: 'uppercase' as const,
    fontWeight: theme.typography.fontWeight.semibold,
    letterSpacing: '0.05em',
  },
  copyButton: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing.xs,
    padding: `${theme.spacing.xs} ${theme.spacing.sm}`,
    fontSize: theme.typography.fontSize.xs,
    backgroundColor: theme.colors.background.secondary,
    border: `1px solid ${theme.colors.border.primary}`,
    borderRadius: theme.borderRadius.sm,
    color: theme.colors.text.secondary,
    cursor: 'pointer',
    transition: `all ${theme.transitions.normal}`,
  },
  copyButtonCopied: {
    backgroundColor: theme.colors.status.success,
    borderColor: theme.colors.status.success,
    color: theme.colors.text.primary,
  },
  copyText: {
    fontSize: theme.typography.fontSize.xs,
    fontWeight: theme.typography.fontWeight.medium,
  },
  checkIcon: {
    color: theme.colors.text.primary,
  },
  syntaxHighlighter: {
    margin: 0,
    padding: theme.spacing.lg,
    backgroundColor: theme.colors.background.primary,
    fontSize: theme.typography.fontSize.sm,
    fontFamily: theme.typography.fontFamily.mono,
  },
};

const inlineCodeStyles = {
  backgroundColor: theme.colors.background.elevated,
  padding: `${theme.spacing.xs} ${theme.spacing.sm}`,
  borderRadius: theme.borderRadius.sm,
  fontSize: '0.9em',
  fontFamily: theme.typography.fontFamily.mono,
  color: theme.colors.text.primary,
  border: `1px solid ${theme.colors.border.primary}`,
};

const paragraphStyles = {
  margin: `${theme.spacing.sm} 0`,
  lineHeight: theme.typography.lineHeight.relaxed,
};

const headingStyles = {
  h1: {
    fontSize: theme.typography.fontSize['3xl'],
    fontWeight: theme.typography.fontWeight.bold,
    margin: `${theme.spacing.lg} 0 ${theme.spacing.md} 0`,
    color: theme.colors.text.primary,
    lineHeight: theme.typography.lineHeight.tight,
  },
  h2: {
    fontSize: theme.typography.fontSize['2xl'],
    fontWeight: theme.typography.fontWeight.semibold,
    margin: `${theme.spacing.md} 0 ${theme.spacing.sm} 0`,
    color: theme.colors.text.primary,
    lineHeight: theme.typography.lineHeight.tight,
  },
  h3: {
    fontSize: theme.typography.fontSize.xl,
    fontWeight: theme.typography.fontWeight.semibold,
    margin: `${theme.spacing.md} 0 ${theme.spacing.sm} 0`,
    color: theme.colors.text.primary,
    lineHeight: theme.typography.lineHeight.tight,
  },
};

const listStyles = {
  ul: {
    margin: `${theme.spacing.sm} 0`,
    paddingLeft: theme.spacing.xl,
  },
  ol: {
    margin: `${theme.spacing.sm} 0`,
    paddingLeft: theme.spacing.xl,
  },
  li: {
    margin: `${theme.spacing.xs} 0`,
    lineHeight: theme.typography.lineHeight.relaxed,
  },
};

const blockquoteStyles = {
  margin: `${theme.spacing.md} 0`,
  padding: `${theme.spacing.sm} ${theme.spacing.lg}`,
  borderLeft: `3px solid ${theme.colors.accent.primary}`,
  backgroundColor: theme.colors.background.elevated,
  borderRadius: theme.borderRadius.sm,
  fontStyle: 'italic' as const,
  color: theme.colors.text.secondary,
};

const linkStyles = {
  color: theme.colors.accent.primary,
  textDecoration: 'none' as const,
  borderBottom: '1px solid transparent',
  transition: `border-color ${theme.transitions.normal}`,
};

const tableWrapperStyles = {
  overflowX: 'auto' as const,
  margin: `${theme.spacing.md} 0`,
};

const tableStyles = {
  width: '100%',
  borderCollapse: 'collapse' as const,
  border: `1px solid ${theme.colors.border.primary}`,
  borderRadius: theme.borderRadius.sm,
  overflow: 'hidden' as const,
};

const tableHeaderStyles = {
  padding: theme.spacing.md,
  backgroundColor: theme.colors.background.elevated,
  borderBottom: `1px solid ${theme.colors.border.primary}`,
  fontWeight: theme.typography.fontWeight.semibold,
  textAlign: 'left' as const,
  color: theme.colors.text.primary,
};

const tableCellStyles = {
  padding: theme.spacing.md,
  borderBottom: `1px solid ${theme.colors.border.primary}`,
  color: theme.colors.text.secondary,
};

// Add hover effect for links
if (!document.getElementById('message-content-styles')) {
  const styleSheet = document.createElement('style');
  styleSheet.id = 'message-content-styles';
  styleSheet.textContent = `
    .message-content-link:hover {
      border-bottom-color: ${theme.colors.accent.primary} !important;
    }
    
    .code-block-copy-button:hover {
      background-color: ${theme.colors.interactive.hover} !important;
      border-color: ${theme.colors.accent.primary} !important;
      transform: translateY(-1px);
    }
    
    .code-block-copy-button:active {
      transform: translateY(0);
    }
  `;
  document.head.appendChild(styleSheet);
}
