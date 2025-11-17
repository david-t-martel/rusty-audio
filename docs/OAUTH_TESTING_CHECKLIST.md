# OAuth Authentication Testing Checklist

## Overview

This comprehensive testing checklist ensures the OAuth 2.0 authentication system is secure, reliable, and provides excellent user experience.

## Pre-Testing Setup

### Environment Configuration

- [ ] OAuth providers configured (Google, GitHub, Microsoft)
- [ ] Redirect URIs registered with providers
- [ ] Client IDs obtained and configured
- [ ] Backend API endpoints deployed
- [ ] HTTPS enabled in production
- [ ] Test accounts created for each provider

### Browser Testing Matrix

Test in the following browsers:

- [ ] Chrome (latest)
- [ ] Firefox (latest)
- [ ] Safari (latest)
- [ ] Edge (latest)
- [ ] Mobile Safari (iOS)
- [ ] Chrome Mobile (Android)

### Test Data

Create test users:
- [ ] Free tier user
- [ ] Premium tier user
- [ ] Suspended/banned user (for error testing)
- [ ] User with expired subscription

## Authentication Flow Testing

### Landing Page

#### Initial Load
- [ ] Landing page loads without errors
- [ ] All OAuth buttons visible and styled correctly
- [ ] Guest mode button visible
- [ ] Features section displays correctly
- [ ] Pricing comparison visible
- [ ] Footer links work

#### Session Check
- [ ] Existing valid session redirects to app
- [ ] Expired session shows landing page
- [ ] Invalid token clears and shows landing page
- [ ] Guest mode persists across page reloads

#### Responsive Design
- [ ] Mobile layout renders correctly
- [ ] Tablet layout renders correctly
- [ ] Desktop layout renders correctly
- [ ] Touch targets are adequately sized (mobile)
- [ ] Text is readable at all breakpoints

### OAuth Initiation

#### Google Sign-In
- [ ] Button click opens popup
- [ ] Popup has correct dimensions (500x600)
- [ ] Authorization URL contains correct parameters
- [ ] State parameter is generated and stored
- [ ] Client ID is correct
- [ ] Redirect URI matches registered URI
- [ ] Scope includes 'openid profile email'

#### GitHub Sign-In
- [ ] Button click opens popup
- [ ] Popup has correct dimensions
- [ ] Authorization URL contains correct parameters
- [ ] State parameter is generated and stored
- [ ] Client ID is correct
- [ ] Redirect URI matches registered URI
- [ ] Scope includes 'read:user user:email'

#### Microsoft Sign-In
- [ ] Button click opens popup
- [ ] Popup has correct dimensions
- [ ] Authorization URL contains correct parameters
- [ ] State parameter is generated and stored
- [ ] Client ID is correct
- [ ] Redirect URI matches registered URI
- [ ] Scope includes 'openid profile email'

#### Loading States
- [ ] Loading overlay appears during auth
- [ ] Loading spinner animates correctly
- [ ] Loading text updates appropriately
- [ ] Buttons disabled during loading
- [ ] Can't trigger multiple auth flows simultaneously

### OAuth Provider Flow

#### Successful Authentication
- [ ] User can authenticate with Google
- [ ] User can authenticate with GitHub
- [ ] User can authenticate with Microsoft
- [ ] Provider redirects to callback URL
- [ ] Callback URL includes authorization code
- [ ] Callback URL includes state parameter
- [ ] No errors in callback URL

#### Authentication Errors
- [ ] User cancels authentication → shows appropriate error
- [ ] User denies permissions → shows appropriate error
- [ ] Invalid redirect URI → handled gracefully
- [ ] Network error during auth → shows error message
- [ ] Provider timeout → shows error message

### OAuth Callback

#### Popup Callback
- [ ] Callback page loads in popup
- [ ] Loading indicator displays
- [ ] Steps progress correctly
- [ ] Code extracted from URL
- [ ] State extracted from URL
- [ ] Error extracted from URL (if present)
- [ ] postMessage sent to parent window
- [ ] Popup closes after success
- [ ] Popup closes after error (with delay)

#### State Validation
- [ ] Valid state passes verification
- [ ] Invalid state throws error
- [ ] Missing state throws error
- [ ] State mismatch detected
- [ ] State cleared after verification
- [ ] CSRF attack prevented

#### Error Handling
- [ ] Missing code handled
- [ ] Missing state handled
- [ ] OAuth provider errors displayed
- [ ] Network errors handled
- [ ] Popup blocker detected
- [ ] User-friendly error messages shown

### Token Exchange

#### Successful Exchange
- [ ] Backend receives authorization code
- [ ] Backend exchanges code for tokens
- [ ] Access token returned
- [ ] Refresh token returned
- [ ] Expires_in value returned
- [ ] User information returned
- [ ] Tokens stored in IndexedDB
- [ ] Tokens encrypted before storage

#### Token Storage
- [ ] IndexedDB database created
- [ ] Object store created
- [ ] Encryption key generated
- [ ] Tokens encrypted with AES-GCM
- [ ] IV included with encrypted data
- [ ] User info encrypted
- [ ] Can retrieve encrypted tokens
- [ ] Can decrypt tokens correctly

#### Exchange Errors
- [ ] Invalid authorization code → error displayed
- [ ] Expired authorization code → error displayed
- [ ] Network error → retry option shown
- [ ] Backend error → error message shown
- [ ] Rate limiting → appropriate message shown

## Session Management

### Session Initialization

- [ ] Session initializes after authentication
- [ ] Activity monitoring starts
- [ ] Multi-tab sync enabled
- [ ] Auto-refresh scheduled
- [ ] Visibility handler registered
- [ ] Session stats available

### Token Refresh

#### Automatic Refresh
- [ ] Refresh triggered 5 minutes before expiration
- [ ] Refresh request sent to backend
- [ ] New access token received
- [ ] New refresh token received
- [ ] Tokens updated in IndexedDB
- [ ] Refresh scheduled for new token
- [ ] Other tabs notified of refresh

#### Manual Refresh
- [ ] Can manually trigger refresh
- [ ] Force refresh works correctly
- [ ] Refresh updates session stats
- [ ] UI updates after refresh

#### Refresh Errors
- [ ] Expired refresh token → logout
- [ ] Invalid refresh token → logout
- [ ] Network error → retry logic
- [ ] Backend error → logout
- [ ] User notified of session expiration

### Activity Monitoring

- [ ] Activity events tracked correctly
- [ ] Last activity time updated
- [ ] Inactivity timer works
- [ ] Inactivity threshold configurable
- [ ] Activity extends session
- [ ] No excessive activity calls (debounced)

### Multi-Tab Synchronization

#### Logout Sync
- [ ] Logout in one tab logs out all tabs
- [ ] localStorage event fires
- [ ] Other tabs receive event
- [ ] Tokens cleared in all tabs
- [ ] All tabs redirect to landing
- [ ] No race conditions

#### Refresh Sync
- [ ] Token refresh in one tab syncs to others
- [ ] Other tabs reload session state
- [ ] No duplicate refresh calls
- [ ] Session stays consistent

#### New Tab Behavior
- [ ] Opening new tab loads existing session
- [ ] New tab announces itself
- [ ] New tab schedules refresh
- [ ] New tab syncs with existing tabs

### Visibility Changes

- [ ] Hidden tab pauses activity monitoring
- [ ] Visible tab resumes monitoring
- [ ] Token validated on visibility
- [ ] Expired token detected when tab visible
- [ ] Refresh triggered if needed

## Feature Gating

### Free Tier

#### Available Features
- [ ] Basic playback accessible
- [ ] Spectrum visualizer accessible
- [ ] Basic EQ accessible
- [ ] Volume control accessible
- [ ] Signal generator accessible
- [ ] Local presets accessible

#### Locked Features
- [ ] Cloud sync shows lock icon
- [ ] AI features show lock icon
- [ ] Advanced effects show lock icon
- [ ] Clicking locked feature shows modal
- [ ] Modal explains feature requirement
- [ ] Modal provides upgrade option

### Authenticated Tier

#### Available Features
- [ ] All free features accessible
- [ ] Cloud sync accessible
- [ ] Usage statistics accessible
- [ ] Profile customization accessible

#### Locked Features
- [ ] Premium features show lock icon
- [ ] Clicking shows upgrade modal
- [ ] Modal explains premium benefits
- [ ] Modal provides upgrade button

### Premium Tier

- [ ] All features accessible
- [ ] No lock icons shown
- [ ] Premium badge displayed
- [ ] Advanced features enabled
- [ ] Priority support mentioned

### Feature Gate UI

- [ ] Lock icons render correctly
- [ ] Feature modals styled properly
- [ ] Upgrade modal dismissible
- [ ] Auth modal dismissible
- [ ] Modal background click closes
- [ ] Escape key closes modal
- [ ] Buttons styled correctly
- [ ] Mobile layout works

## User Interface

### Loading States

- [ ] Initial page load smooth
- [ ] Auth button loading states
- [ ] Overlay loading animation
- [ ] Progress indicators work
- [ ] Loading text updates
- [ ] Skeleton screens (if used)

### Error States

#### Error Alert
- [ ] Error alert slides down
- [ ] Error message displayed
- [ ] Alert auto-dismisses after 5s
- [ ] Can manually dismiss
- [ ] Multiple errors queued
- [ ] Error icon shown

#### Error Messages
- [ ] Network error: user-friendly message
- [ ] Auth error: clear explanation
- [ ] Token error: action required message
- [ ] Rate limit: wait time displayed
- [ ] Popup blocked: instructions shown

### Success States

- [ ] Successful auth redirects to app
- [ ] Session restored smoothly
- [ ] User avatar loads
- [ ] User name displayed
- [ ] Tier badge shown
- [ ] Welcome message (optional)

### Responsive Design

#### Mobile (< 768px)
- [ ] Auth buttons full width
- [ ] Text readable
- [ ] Touch targets adequate (44x44px)
- [ ] No horizontal scroll
- [ ] Popup still works
- [ ] Modals fit screen

#### Tablet (768px - 1024px)
- [ ] Layout adapts correctly
- [ ] Images scale properly
- [ ] Grid layouts work
- [ ] Modals sized appropriately

#### Desktop (> 1024px)
- [ ] Full feature set accessible
- [ ] Optimal layout used
- [ ] Hover states work
- [ ] Keyboard navigation works

## Security Testing

### Token Security

- [ ] Tokens never in localStorage
- [ ] Tokens never in sessionStorage
- [ ] Tokens encrypted in IndexedDB
- [ ] Encryption key secure
- [ ] No tokens in URL
- [ ] No tokens in console.log
- [ ] No tokens in error messages

### CSRF Protection

- [ ] State parameter generated securely
- [ ] State stored in sessionStorage
- [ ] State validated on callback
- [ ] Invalid state rejected
- [ ] State cleared after use
- [ ] Can't replay old state

### XSS Prevention

- [ ] User input sanitized
- [ ] No innerHTML with user data
- [ ] CSP headers configured
- [ ] No eval() usage
- [ ] No Function() with user input
- [ ] Framework escaping works

### Origin Validation

- [ ] postMessage origin validated
- [ ] Only same-origin accepted
- [ ] Cross-origin rejected
- [ ] Warning logged for invalid origins

### HTTPS Enforcement

- [ ] Production uses HTTPS
- [ ] HTTP redirects to HTTPS
- [ ] Mixed content prevented
- [ ] All resources secure
- [ ] OAuth flows over HTTPS

## Error Recovery

### Network Errors

- [ ] Retry button shown
- [ ] Retry works correctly
- [ ] Exponential backoff implemented
- [ ] Max retries enforced
- [ ] User can cancel retry

### Token Expiration

- [ ] Expired token detected
- [ ] Refresh attempted automatically
- [ ] User notified if refresh fails
- [ ] Logout on failed refresh
- [ ] Redirect to landing page

### Session Corruption

- [ ] Invalid session cleared
- [ ] User notified
- [ ] Fresh login prompted
- [ ] No infinite loops
- [ ] Error logged

### Browser Storage Issues

- [ ] IndexedDB unavailable → fallback
- [ ] Storage quota exceeded → handled
- [ ] Private browsing detected
- [ ] User notified of limitations

## Performance Testing

### Page Load

- [ ] Landing page loads < 2s
- [ ] JavaScript bundle optimized
- [ ] CSS minified
- [ ] Images optimized
- [ ] No render-blocking resources
- [ ] Lighthouse score > 90

### Authentication Flow

- [ ] Auth initiation < 500ms
- [ ] Token exchange < 1s
- [ ] Session init < 500ms
- [ ] Total auth time < 3s
- [ ] No janky animations
- [ ] Smooth transitions

### Memory Usage

- [ ] No memory leaks
- [ ] Event listeners cleaned up
- [ ] Intervals cleared
- [ ] Timers cleared
- [ ] Objects garbage collected
- [ ] Memory usage stable

### Network Usage

- [ ] Minimal API calls
- [ ] Responses cached appropriately
- [ ] No redundant requests
- [ ] Batch requests where possible
- [ ] Compression enabled

## Accessibility

### Keyboard Navigation

- [ ] Can tab through all interactive elements
- [ ] Tab order logical
- [ ] Focus visible
- [ ] Can activate with Enter/Space
- [ ] Escape closes modals
- [ ] No keyboard traps

### Screen Reader

- [ ] Alt text on images
- [ ] ARIA labels on buttons
- [ ] ARIA live regions for updates
- [ ] Semantic HTML used
- [ ] Headings hierarchical
- [ ] Form labels associated

### Visual Accessibility

- [ ] Color contrast meets WCAG AA
- [ ] Focus indicators visible
- [ ] Error states clear
- [ ] Text scalable
- [ ] No information by color alone

### Motion Accessibility

- [ ] Reduced motion respected
- [ ] Animations can be disabled
- [ ] No auto-playing videos
- [ ] No flashing content

## Edge Cases

### Unusual Flows

- [ ] Back button during auth
- [ ] Refresh during auth
- [ ] Close popup manually
- [ ] Multiple popups opened
- [ ] Auth in incognito mode
- [ ] Auth with ad blockers

### Concurrent Operations

- [ ] Multiple tabs auth simultaneously
- [ ] Multiple refresh attempts
- [ ] Logout during refresh
- [ ] Auth during logout
- [ ] Session expiry during use

### Browser Quirks

- [ ] Safari third-party cookies
- [ ] Firefox tracking protection
- [ ] Edge compatibility
- [ ] iOS Safari popup handling
- [ ] Android Chrome behavior

## Post-Deployment Testing

### Production Verification

- [ ] OAuth providers work in production
- [ ] Callback URLs configured correctly
- [ ] HTTPS enforced
- [ ] CSP headers active
- [ ] All endpoints reachable
- [ ] No console errors

### Monitoring

- [ ] Authentication success rate > 95%
- [ ] Token refresh success rate > 99%
- [ ] Average auth time < 3s
- [ ] Error rate < 1%
- [ ] No security alerts

### User Acceptance

- [ ] User feedback positive
- [ ] No major usability issues
- [ ] Performance acceptable
- [ ] Mobile experience smooth
- [ ] Support tickets minimal

## Regression Testing

Run this checklist:
- [ ] After code changes
- [ ] After dependency updates
- [ ] After OAuth provider changes
- [ ] After security updates
- [ ] Monthly in production

## Test Automation

### Unit Tests

```javascript
// Example test structure
describe('AuthManager', () => {
  test('generates secure state parameter', () => {
    const state = authManager.generateState();
    expect(state).toHaveLength(43); // Base64 URL encoded 32 bytes
    expect(state).toMatch(/^[A-Za-z0-9_-]+$/);
  });

  test('validates state correctly', () => {
    const state = authManager.generateState();
    sessionStorage.setItem('oauth_state', state);
    expect(() => authManager.verifyState(state)).not.toThrow();
    expect(() => authManager.verifyState('invalid')).toThrow();
  });
});
```

### Integration Tests

```javascript
describe('OAuth Flow', () => {
  test('complete Google auth flow', async () => {
    // 1. Initiate auth
    const authPromise = authManager.signInWithGoogle();

    // 2. Simulate callback
    window.postMessage({
      code: 'test_code',
      state: sessionStorage.getItem('oauth_state')
    }, window.location.origin);

    // 3. Verify tokens stored
    await authPromise;
    const token = await tokenStorage.getAccessToken();
    expect(token).toBeTruthy();
  });
});
```

### E2E Tests

```javascript
describe('End-to-End Auth', () => {
  test('user can sign in and access premium features', async () => {
    // 1. Visit landing page
    await page.goto('https://rusty-audio.com');

    // 2. Click Google sign-in
    await page.click('[data-testid="google-signin"]');

    // 3. Handle OAuth popup
    const popup = await page.waitForEvent('popup');
    await popup.fill('[name="email"]', 'test@example.com');
    await popup.fill('[name="password"]', 'password');
    await popup.click('[type="submit"]');

    // 4. Wait for redirect
    await page.waitForURL('**/app');

    // 5. Verify premium features accessible
    const premiumFeature = page.locator('[data-feature="ai_enhancement"]');
    await expect(premiumFeature).not.toHaveClass(/locked/);
  });
});
```

## Sign-Off

### Development Team

- [ ] All tests passed
- [ ] Code reviewed
- [ ] Security reviewed
- [ ] Performance acceptable
- [ ] Documentation updated

### QA Team

- [ ] Manual testing complete
- [ ] Automated tests passing
- [ ] No critical bugs
- [ ] No major usability issues
- [ ] Ready for deployment

### Security Team

- [ ] Security audit complete
- [ ] No vulnerabilities found
- [ ] Penetration testing done
- [ ] Compliance verified
- [ ] Approved for production

---

**Test Date**: _______________

**Tested By**: _______________

**Version**: _______________

**Status**: ☐ Pass ☐ Fail ☐ Blocked

**Notes**:
_______________________________________
_______________________________________
_______________________________________
