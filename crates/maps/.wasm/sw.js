var cacheName = 'maps-pwa';
var filesToCache = [
  './',
  './index.html',
];

/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function (e) {
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    caches.match(e.request).then(function (response) {
      if (response) {
        return response;
      }
      // Fetch and cache the response for future use
      return fetch(e.request).then(function(response) {
        // Check if we received a valid response
        if (!response || response.status !== 200 || response.type !== 'basic') {
          return response;
        }
        // Clone the response as it can only be consumed once
        var responseToCache = response.clone();
        caches.open(cacheName).then(function(cache) {
          cache.put(e.request, responseToCache);
        });
        return response;
      });
    })
  );
});
