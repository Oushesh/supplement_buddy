"""WSGI config for supplement_buddy project."""

import os

from django.core.wsgi import get_wsgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "supplement_buddy.settings")

application = get_wsgi_application()
