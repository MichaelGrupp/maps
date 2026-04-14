var cacheName = 'maps-pwa-__CACHE_VERSION__';
var filesToCache = [
  './',
  './index.html',
];

/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function (e) {
  self.skipWaiting();
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Delete old caches when a new service worker activates */
self.addEventListener('activate', function (e) {
  e.waitUntil(
    caches.keys().then(function (cacheNames) {
      return Promise.all(
        cacheNames.map(function (name) {
          if (name !== cacheName) {
            return caches.delete(name);
          }
        })
      );
    }).then(function () {
      return self.clients.claim();
    })
  );
});

/* Network-first strategy: try network, fall back to cache for offline use */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    fetch(e.request).then(function (response) {
      if (!response || response.status !== 200 || response.type !== 'basic') {
        return response;
      }
      var responseToCache = response.clone();
      caches.open(cacheName).then(function (cache) {
        cache.put(e.request, responseToCache);
      });
      return response;
    }).catch(function () {
      return caches.match(e.request);
    })
  );
});
