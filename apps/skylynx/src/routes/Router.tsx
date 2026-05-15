import { lazy } from 'solid-js';
import { Route, Router } from '@solidjs/router';
import { AppShell } from '../components/AppShell';
import { AuthProvider } from '../lib/auth/AuthContext';

const LinkDigestRoute = lazy(() => import('./App').then((m) => ({ default: m.LinkDigest })));

const AboutRoute = lazy(() => import('./About').then((m) => ({ default: m.About })));

function App() {
  return (
    <AuthProvider>
      <Router root={AppShell}>
        <Route path="/" component={AboutRoute} />
        <Route path="/app" component={LinkDigestRoute} />
        <Route path="/oauth/callback" component={LinkDigestRoute} />
      </Router>
    </AuthProvider>
  );
}

export default App;
