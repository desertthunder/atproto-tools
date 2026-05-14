import { lazy } from 'solid-js';
import { Route, Router } from '@solidjs/router';
import { AppShell } from './components/AppShell';

const LinkDigestRoute = lazy(() =>
  import('./routes/LinkDigest').then((m) => ({ default: m.LinkDigest }))
);

const AboutRoute = lazy(() =>
  import('./routes/About').then((m) => ({ default: m.About }))
);

function App() {
  return (
    <Router root={AppShell}>
      <Route path="/" component={AboutRoute} />
      <Route path="/app" component={LinkDigestRoute} />
    </Router>
  );
}

export default App;
