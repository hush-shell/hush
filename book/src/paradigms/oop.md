# Object Oriented

*Hush* provides basic support for object oriented programming. This means you can write object oriented code in *Hush*, but the language won't give you fancy features for free. Objects can be modeled through dictionaries, using keys as accessors and values as member data and methods. To ease the use of dictionaries as objects, *Hush* provides the `self` keyword, which is described in the [functions section](../intro/control-flow.md#functions).

```hush
# This function acts like a combination of a class and a constructor. It'll take any
# arguments relevant to the construction of a `MyCounter` object, and will return an
# instance of such object, which is nothing but a dictionary.
let MyCounter = function(start)
	@[
		# A member field. Using the same convention as Python, a field starting with an
		# underscore should be considered private.
		_start: start,

		# A method. Here, we can use the `self` keyword to access the object.
		get: function()
			self._start
		end,

		# Another method.
		increment: function()
			self._start = self._start + 1
		end,
	 ]
end

let counter = MyCounter(1) # object instantiation.

std.print(counter.get()) # 1
counter.increment()
std.print(counter.get()) # 2
```

## Inheritance

Single inheritance can be implemented using a similar technique:

```hush
# A derived class.
let MySuperCounter = function (start, step)
	# Instantiate an object of the base class. We'll then augment this object with
	# derived functionality.
	let counter = MyCounter(start)

	counter._step = step # Add new fields to the object.

	# Override a method. Make sure not to change the number of parameters here!
	counter.increment = function ()
		self._start = self._start + self._step
	end

	# In order to override a method and call the parent implementation, you'll need to
	# bind it to the current object, and then store it to a variable:
	let super_get = std.bind(counter, counter.get)
	counter.get = function()
		let value = super_get() # call the parent method.
		std.print(value)
		value
	end

	counter
end


let super_counter = MySuperCounter(2, 3)
super_counter.get() # 2
super_counter.increment()
super_counter.get() # 5
```
