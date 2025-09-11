document.addEventListener('DOMContentLoaded', function() {
  // Mobile menu toggle
  const menuButton = document.getElementById('menuButton');
  const mobileMenu = document.getElementById('mobileMenu');
  
  if (menuButton && mobileMenu) {
    menuButton.addEventListener('click', function() {
      mobileMenu.classList.toggle('show');
    });
  }
  
  // Smooth scrolling for navigation links
  const navLinks = document.querySelectorAll('a[href^="#"]');
  
  navLinks.forEach(link => {
    link.addEventListener('click', function(e) {
      const targetId = this.getAttribute('href');
      
      if (targetId === '#') return;
      
      e.preventDefault();
      
      const targetElement = document.querySelector(targetId);
      
      if (targetElement) {
        // Close mobile menu if open
        if (mobileMenu && mobileMenu.classList.contains('show')) {
          mobileMenu.classList.remove('show');
        }
        
        // Scroll to target
        window.scrollTo({
          top: targetElement.offsetTop - 80, // Adjust for header height
          behavior: 'smooth'
        });
      }
    });
  });
  
  // // Form submission handling
  // const contactForm = document.querySelector('.contact-form');
  
  // if (contactForm) {
  //   contactForm.addEventListener('submit', function(e) {
  //     e.preventDefault();
      
  //     // Get form data
  //     const formData = {
  //       name: document.getElementById('name').value,
  //       email: document.getElementById('email').value,
  //       subject: document.getElementById('subject').value,
  //       message: document.getElementById('message').value
  //     };
      
  //     // Here you would typically send the data to a server
  //     console.log('Form submitted:', formData);
      
  //     // Show success message (in a real implementation, this would happen after successful submission)
  //     const successMessage = document.createElement('div');
  //     successMessage.className = 'success-message';
  //     successMessage.textContent = 'Message sent successfully!';
      
  //     contactForm.appendChild(successMessage);
      
  //     // Reset form
  //     contactForm.reset();
      
  //     // Remove success message after 3 seconds
  //     setTimeout(() => {
  //       successMessage.remove();
  //     }, 3000);
  //   });
  // }
  
  // Active navigation highlighting based on scroll position
  function highlightActiveNavLink() {
    const sections = document.querySelectorAll('section[id]');
    const scrollPosition = window.scrollY;
    
    sections.forEach(section => {
      const sectionTop = section.offsetTop - 100;
      const sectionHeight = section.offsetHeight;
      const sectionId = section.getAttribute('id');
      
      if (scrollPosition >= sectionTop && scrollPosition < sectionTop + sectionHeight) {
        // Remove active class from all links
        navLinks.forEach(link => {
          link.classList.remove('active');
        });
        
        // Add active class to current section link
        const activeLink = document.querySelector(`a[href="#${sectionId}"]`);
        if (activeLink) {
          activeLink.classList.add('active');
        }
      }
    });
  }
  
  window.addEventListener('scroll', highlightActiveNavLink);
  
  // Update copyright year
  const copyrightYear = document.querySelector('.copyright-year');
  if (copyrightYear) {
    copyrightYear.textContent = new Date().getFullYear();
  }
}); 