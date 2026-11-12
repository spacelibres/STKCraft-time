import importlib.util
import unittest

class BoundaryTests(unittest.TestCase):
    def test_extension_package_exists(self):
        self.assertIsNotNone(importlib.util.find_spec('stkcraft_time'), 'native Python package must be installed')

if __name__ == '__main__': unittest.main()
